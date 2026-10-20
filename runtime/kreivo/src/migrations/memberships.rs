//! `MembershipsToGroupCollections` (M-3 `F-10`, PLAN §6.3, `DEC-30`, `DEC-33`, `DEC-35`).
//!
//! At the 0.18 upgrade the live memberships move into their groups' own collections: the store
//! (collection 0) is retired, its stock converted, every membership locked, indexed and backed by a
//! deposit, the collections handed to the memberships manager, the admin-held store items refunded,
//! and each community's storage part held.
//!
//! **It must pass** (owner, 2026-10-19; `T-10-00`). A step never returns an error because of the
//! data: an entry that doesn't fit is skipped and logged with `log::warn!`, and the hard checks live
//! in [`MembershipsToGroupCollections::pre_upgrade_checks`] and
//! [`MembershipsToGroupCollections::post_upgrade_checks`] (try-runtime). The only error a step
//! returns is `InsufficientWeight`, when not even one unit of the current phase fits an empty block.
//!
//! **Every phase yields.** A phase walks its keys in storage order, from the raw key its cursor
//! holds. It computes one unit's weight, checks that `meter.remaining()` holds one unit, and then
//! loops: `meter.try_consume(unit)` (on failure it yields, with the cursor at the last key done),
//! one atomic unit of work, advance the cursor. Each unit runs in its own storage layer, so a unit
//! that is refused changes nothing.

use super::*;

use crate::{
	config::{
		communities::{
			memberships::CommunityMembershipsInstance, CommunityPalletId, CommunityStoragePartAmount,
			MembershipDepositAmount, MembershipsManagerAccount,
		},
		governance::TreasuryAccount,
	},
	Balances, Communities, CommunitiesManager, CommunityMemberships, RuntimeHoldReason, RuntimeOrigin,
};
use alloc::{vec, vec::Vec};
use frame_contrib_traits::memberships::{Inspect as _, Issue, Rank};
use frame_support::{
	migrations::{SteppedMigration, SteppedMigrationError},
	pallet_prelude::*,
	storage::{storage_prefix, with_storage_layer},
	traits::{
		fungible::{hold::Mutate as _, Inspect as _, InspectHold as _, Mutate as _},
		nonfungibles_v2 as nfv2,
		tokens::{Fortitude, Preservation},
		GetStorageVersion, StorageVersion,
	},
	weights::WeightMeter,
	Blake2_128Concat, StorageHasher,
};
use pallet_communities_manager::{CommunityDeposits, Deposits, HoldReason, NextMembershipId};
use pallet_nfts::{
	Attribute, AttributeNamespace, Collection, CollectionAccount, CollectionConfigOf, CollectionRoleOf,
	CollectionSetting, ItemConfig, ItemConfigOf, ItemPriceOf, ItemSetting,
};
use sp_runtime::traits::{AccountIdConversion, StaticLookup, Zero};
use virto_common::{CommunityId, MembershipId};

type I = CommunityMembershipsInstance;
type ItemOf = pallet_nfts::Item<Runtime, I>;
type NftsPallet = pallet_nfts::Pallet<Runtime, I>;
type KeyLimit = <Runtime as pallet_nfts::Config<I>>::KeyLimit;
type AttributeKey = (
	CommunityId,
	Option<MembershipId>,
	AttributeNamespace<AccountId>,
	BoundedVec<u8, KeyLimit>,
);

/// The id of the store collection, which this migration retires.
const STORE: CommunityId = 0;

/// The version `communities-manager`'s storage reaches when this migration completes.
const TARGET_VERSION: StorageVersion = StorageVersion::new(1);

/// The attribute keys a per-membership gas tank wrote (`REQ-RT-2`), prefixed or not.
const TANK_KEYS: [&[u8]; 3] = [b"membership_gas", b"mbmshp_pays_gas", b"membership_expiration"];

/// The longest raw storage key a cursor holds: an `Attribute` key is 32 bytes of prefix, then the
/// four hashed parts (at most 20, 21, 33 and 81 bytes).
const MAX_RAW_KEY: u32 = 256;

/// The refunds of the admin-held store items (`DEP-18`, 0007-A2), as `(item, community, amount)`:
/// item 100 at 1.0 KSM and six at 0.3 KSM, 2.8 KSM in all. The holder of each is the signed admin of
/// the community (`CommunityIdFor`), as `NOTES.md` §D.46 lists them; `pre_upgrade` fails on a row
/// that no longer matches, and on chain a mismatched row is skipped and logged.
pub const REFUNDS: [(MembershipId, CommunityId, Balance); 7] = [
	(10_064, 777, 3 * UNITS / 10),
	(10_441, 777, 3 * UNITS / 10),
	(100, 1_107, UNITS),
	(10_161, 69, 3 * UNITS / 10),
	(10_071, 1_786, 3 * UNITS / 10),
	(10_234, 1_786, 3 * UNITS / 10),
	(10_393, 1_786, 3 * UNITS / 10),
];

/// A store item that M4 burnt from its listed holder, whose refund M9 pays. M4 records it, because
/// by M9 the item is gone and so is the proof of who held it.
#[frame_support::storage_alias(verbatim)]
pub type PendingRefunds = StorageMap<MembershipsMigration, Blake2_128Concat, MembershipId, (AccountId, Balance)>;

/// A registered community, as `pallet_communities` keeps it (its storage is private).
#[frame_support::storage_alias]
type Info = StorageMap<Communities, Blake2_128Concat, CommunityId, ()>;

/// Which origin is a community's admin, as `pallet_communities` keeps it (its storage is private).
#[frame_support::storage_alias]
type CommunityIdFor = StorageMap<
	Communities,
	Blake2_128Concat,
	<RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin,
	CommunityId,
>;

/// The phases, in the order PLAN §6.3 runs them (M0, M1, M2, M3, M4, M5, M6, M8, M9, M10, M11, M7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Phase {
	/// M0: reads the attribute keys, and logs a tank or expiration key (`pre_upgrade` fails on one).
	Guard,
	/// M1: `NextMembershipId` above every id that has ever existed.
	Ids,
	/// M2: converts the stock the communities hold in the store into their own collections.
	Stock,
	/// M3: locks every membership.
	Lock,
	/// M4: retires the store.
	Retire,
	/// M5: clears the attributes of items that no longer exist.
	Orphans,
	/// M6: a deposit for every membership, from the Treasury.
	Deposits,
	/// M8: hands the collections to the memberships manager.
	Manager,
	/// M9: refunds the admin-held store items.
	Refunds,
	/// M10: indexes every membership.
	Index,
	/// M11: holds each community's storage part.
	StorageParts,
	/// M7: sets the storage version.
	Version,
}

impl Phase {
	/// The phase after this one.
	pub fn next(self) -> Option<Self> {
		use Phase::*;
		Some(match self {
			Guard => Ids,
			Ids => Stock,
			Stock => Lock,
			Lock => Retire,
			Retire => Orphans,
			Orphans => Deposits,
			Deposits => Manager,
			Manager => Refunds,
			Refunds => Index,
			Index => StorageParts,
			StorageParts => Version,
			Version => return None,
		})
	}

	/// The weight of one unit of the phase, a generous bound of what the unit reads and writes
	/// (PLAN §6.4: `do_burn` ≈ 4 reads and 6 writes, `do_mint` ≈ 4 and 4, a system attribute ≈ 2 and 2).
	pub fn unit(self) -> Weight {
		use Phase::*;
		match self {
			Guard => unit(1, 0),
			Ids => unit(2, 1),
			Stock => unit(14, 14),
			Lock => unit(4, 2),
			Retire => unit(8, 10),
			Orphans => unit(5, 4),
			Deposits => unit(8, 8),
			Manager => unit(8, 8),
			Refunds => unit(6, 6),
			Index => unit(4, 1),
			StorageParts => unit(8, 8),
			Version => unit(1, 1),
		}
	}
}

/// What the migration remembers between steps: the phase, and the last raw storage key it finished
/// in it (`None`: it starts the phase).
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct Cursor {
	pub phase: Phase,
	pub last: Option<BoundedVec<u8, ConstU32<MAX_RAW_KEY>>>,
}

impl Cursor {
	/// The cursor at the start of `phase`.
	pub fn start_of(phase: Phase) -> Self {
		Self { phase, last: None }
	}
}

/// Estimated proof size of one storage access, in bytes: 300–600 for the value and the trie nodes
/// (PLAN §6.4), doubled.
const PROOF_PER_ACCESS: u64 = 1_000;
/// What the unit's code costs besides its storage accesses, in picoseconds of `ref_time`.
const EXECUTION_REF_TIME: u64 = 50_000_000;

fn unit(reads: u64, writes: u64) -> Weight {
	<Runtime as frame_system::Config>::DbWeight::get()
		.reads_writes(reads, writes)
		.saturating_add(Weight::from_parts(
			EXECUTION_REF_TIME,
			(reads + writes).saturating_mul(PROOF_PER_ACCESS),
		))
}

/// The heaviest unit of any phase. It has to fit `MaxServiceWeight` (`T-10-00`).
pub fn heaviest_unit() -> Weight {
	let mut phase = Some(Phase::Guard);
	let mut heaviest = Weight::zero();
	while let Some(p) = phase {
		heaviest = heaviest.max(p.unit());
		phase = p.next();
	}
	heaviest
}

/// Moves the live memberships into their groups' collections, in steps.
pub struct MembershipsToGroupCollections;

pub(crate) enum Outcome {
	/// The phase has no more keys.
	Done,
	/// The block's weight ran out: the last raw key finished.
	Yield(Option<Vec<u8>>),
}

/// One phase's loop (the PSDK yield pattern): consume one unit, or yield; do one atomic unit, and
/// advance the cursor. `next` finds the key after the cursor, `apply` does the unit.
fn drive<T>(
	meter: &mut WeightMeter,
	unit: Weight,
	mut last: Option<Vec<u8>>,
	next: impl Fn(&Option<Vec<u8>>) -> Option<(Vec<u8>, T)>,
	mut apply: impl FnMut(T),
) -> Outcome {
	loop {
		if meter.try_consume(unit).is_err() {
			return Outcome::Yield(last);
		}
		match next(&last) {
			None => return Outcome::Done,
			Some((raw, item)) => {
				apply(item);
				last = Some(raw);
			}
		}
	}
}

/// Runs one unit in its own storage layer: a refusal rolls it back, and is logged, never raised.
fn atomic(what: &str, unit: impl FnOnce() -> DispatchResult) {
	if let Err(error) = with_storage_layer(unit) {
		log::warn!(target: LOG_TARGET, "memberships migration: {what} skipped: {error:?}");
	}
}

fn next_item(last: &Option<Vec<u8>>) -> Option<(Vec<u8>, (CommunityId, MembershipId))> {
	let mut keys = match last {
		Some(raw) => ItemOf::iter_keys_from(raw.clone()),
		None => ItemOf::iter_keys(),
	};
	keys.next()
		.map(|(collection, item)| (ItemOf::hashed_key_for(collection, item), (collection, item)))
}

fn next_store_item(last: &Option<Vec<u8>>) -> Option<(Vec<u8>, MembershipId)> {
	let mut items = match last {
		Some(raw) => ItemOf::iter_prefix_from(STORE, raw.clone()),
		None => ItemOf::iter_prefix(STORE),
	};
	items
		.next()
		.map(|(item, _)| (ItemOf::hashed_key_for(STORE, item), item))
}

fn next_attribute(last: &Option<Vec<u8>>) -> Option<(Vec<u8>, AttributeKey)> {
	let mut keys = match last {
		Some(raw) => Attribute::<Runtime, I>::iter_keys_from(raw.clone()),
		None => Attribute::<Runtime, I>::iter_keys(),
	};
	keys.next()
		.map(|key| (Attribute::<Runtime, I>::hashed_key_for(key.clone()), key))
}

fn next_collection(last: &Option<Vec<u8>>) -> Option<(Vec<u8>, CommunityId)> {
	let mut keys = match last {
		Some(raw) => Collection::<Runtime, I>::iter_keys_from(raw.clone()),
		None => Collection::<Runtime, I>::iter_keys(),
	};
	keys.next()
		.map(|collection| (Collection::<Runtime, I>::hashed_key_for(collection), collection))
}

fn next_community(last: &Option<Vec<u8>>) -> Option<(Vec<u8>, CommunityId)> {
	let mut keys = match last {
		Some(raw) => Info::iter_keys_from(raw.clone()),
		None => Info::iter_keys(),
	};
	keys.next()
		.map(|community| (Info::hashed_key_for(community), community))
}

fn next_pending_refund(last: &Option<Vec<u8>>) -> Option<(Vec<u8>, (MembershipId, AccountId, Balance))> {
	let mut entries = match last {
		Some(raw) => PendingRefunds::iter_from(raw.clone()),
		None => PendingRefunds::iter(),
	};
	entries
		.next()
		.map(|(item, (holder, amount))| (PendingRefunds::hashed_key_for(item), (item, holder, amount)))
}

/// The community whose account `who` is, if it is one.
fn community_of(who: &AccountId) -> Option<CommunityId> {
	let (pallet, community) = PalletId::try_from_sub_account::<CommunityId>(who)?;
	(pallet == CommunityPalletId::get()
		&& Communities::community_account(&community) == *who
		&& Info::contains_key(community))
	.then_some(community)
}

/// Whether `who` is the signed admin of `community`.
fn is_admin_of(who: &AccountId, community: CommunityId) -> bool {
	let origin: <RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin =
		frame_system::RawOrigin::Signed(who.clone()).into();
	CommunityIdFor::get(origin) == Some(community)
}

fn refund_row(item: MembershipId) -> Option<(CommunityId, Balance)> {
	REFUNDS
		.iter()
		.find(|(id, _, _)| *id == item)
		.map(|(_, community, amount)| (*community, *amount))
}

/// The holder of an item.
fn owner_of(collection: CommunityId, item: MembershipId) -> Option<AccountId> {
	NftsPallet::owner(collection, item)
}

fn treasury() -> AccountId {
	TreasuryAccount::get()
}

fn hold_reason(reason: HoldReason) -> RuntimeHoldReason {
	RuntimeHoldReason::CommunitiesManager(reason)
}

/// Where `GroupCollectionMemberships` indexes a membership's group (frame-contrib 2.4.0).
pub fn membership_index_key(m: &MembershipId) -> Vec<u8> {
	let mut key = storage_prefix(b"GroupCollectionMemberships", b"GroupOf").to_vec();
	key.extend(Blake2_128Concat::hash(&m.encode()));
	key
}

/// A transfer from the Treasury to `to`, held there under `reason`.
fn fund_and_hold(to: &AccountId, amount: Balance, reason: HoldReason) -> DispatchResult {
	Balances::transfer(&treasury(), to, amount, Preservation::Preserve)?;
	Balances::hold(&hold_reason(reason), to, amount)?;
	Ok(())
}

impl MembershipsToGroupCollections {
	/// One phase, as far as the meter goes from `last` (the PSDK yield pattern, see the module).
	pub(crate) fn run(phase: Phase, meter: &mut WeightMeter, last: Option<Vec<u8>>) -> Outcome {
		let unit = phase.unit();
		match phase {
			// M0
			Phase::Guard => drive(meter, unit, last, next_attribute, |(collection, item, _, key)| {
				if TANK_KEYS.iter().any(|tank| key.ends_with(tank)) {
					log::warn!(
						target: LOG_TARGET,
						"memberships migration: a gas tank or expiration attribute remains on ({collection}, {item:?})"
					);
				}
			}),
			// M1: the highest id so far lives in `NextMembershipId` itself.
			Phase::Ids => drive(meter, unit, last, next_item, |(_, item)| {
				let next = item.saturating_add(1);
				if NextMembershipId::<Runtime>::get() < next {
					NextMembershipId::<Runtime>::put(next);
				}
			}),
			// M2
			Phase::Stock => drive(meter, unit, last, next_store_item, |item| {
				let Some(holder) = owner_of(STORE, item) else {
					return;
				};
				let Some(community) = community_of(&holder) else {
					return;
				};
				atomic("a stock item", || {
					<CommunityMemberships as nfv2::Mutate<AccountId, ItemConfig>>::burn(&STORE, &item, None)?;
					<CommunityMemberships as nfv2::Mutate<AccountId, ItemConfig>>::mint_into(
						&community,
						&item,
						&holder,
						&ItemConfig::default(),
						false,
					)?;
					<crate::Memberships as Issue<AccountId>>::lock(&community, &item)
				});
			}),
			// M3
			Phase::Lock => drive(meter, unit, last, next_item, |(collection, item)| {
				if collection != STORE {
					atomic("a lock", || {
						<crate::Memberships as Issue<AccountId>>::lock(&collection, &item)
					});
				}
			}),
			// M4: the attributes the burn leaves behind are M5's orphans.
			Phase::Retire => drive(meter, unit, last, next_store_item, |item| {
				let holder = owner_of(STORE, item);
				atomic("a store item", || {
					<CommunityMemberships as nfv2::Mutate<AccountId, ItemConfig>>::burn(&STORE, &item, None)?;
					if let (Some(holder), Some((community, amount))) = (holder, refund_row(item)) {
						if is_admin_of(&holder, community) {
							PendingRefunds::insert(item, (holder, amount));
						} else {
							log::warn!(
								target: LOG_TARGET,
								"memberships migration: store item {item} is not held by its community's admin, no refund"
							);
						}
					}
					Ok(())
				});
			}),
			// M5
			Phase::Orphans => drive(
				meter,
				unit,
				last,
				next_attribute,
				|(collection, item, namespace, key)| {
					if let Some(item) = item {
						if !ItemOf::contains_key(collection, item) {
							atomic("an orphan attribute", || {
								NftsPallet::clear_attribute(
									RuntimeOrigin::root(),
									collection,
									Some(item),
									namespace,
									key,
								)
							});
						}
					}
				},
			),
			// M6
			Phase::Deposits => drive(meter, unit, last, next_item, |(collection, item)| {
				if collection == STORE || Deposits::<Runtime>::contains_key((collection, item)) {
					return;
				}
				atomic("a membership deposit", || {
					let deposit = MembershipDepositAmount::get();
					fund_and_hold(
						&Communities::community_account(&collection),
						deposit,
						HoldReason::MembershipDeposit,
					)?;
					Deposits::<Runtime>::insert((collection, item), deposit);
					Ok(())
				});
			}),
			// M8
			Phase::Manager => drive(meter, unit, last, next_collection, |collection| {
				let manager = MembershipsManagerAccount::get();
				if Collection::<Runtime, I>::get(collection).is_some_and(|details| details.owner == manager) {
					return;
				}
				atomic("a collection's hand-over", || {
					let source = <<Runtime as frame_system::Config>::Lookup as StaticLookup>::unlookup(manager.clone());
					NftsPallet::force_collection_owner(RuntimeOrigin::root(), collection, source.clone())?;
					NftsPallet::set_team(
						RuntimeOrigin::root(),
						collection,
						Some(source.clone()),
						Some(source.clone()),
						Some(source),
					)
				});
			}),
			// M9
			Phase::Refunds => drive(meter, unit, last, next_pending_refund, |(item, holder, amount)| {
				PendingRefunds::remove(item);
				atomic("a refund", || {
					Balances::transfer(&treasury(), &holder, amount, Preservation::Preserve).map(|_| ())
				});
			}),
			// M10
			Phase::Index => drive(meter, unit, last, next_item, |(collection, item)| {
				if collection != STORE {
					atomic("an index entry", || {
						<crate::Memberships as Issue<AccountId>>::index(&collection, &item)
					});
				}
			}),
			// M11
			Phase::StorageParts => drive(meter, unit, last, next_community, |community| {
				let part = CommunityStoragePartAmount::get();
				if part.is_zero() || CommunityDeposits::<Runtime>::contains_key(community) {
					return;
				}
				atomic("a storage part", || {
					fund_and_hold(
						&Communities::community_account(&community),
						part,
						HoldReason::CommunityDeposit,
					)?;
					CommunityDeposits::<Runtime>::insert(community, part);
					Ok(())
				});
			}),
			// M7
			Phase::Version => drive(
				meter,
				unit,
				last,
				|last| last.is_none().then(|| (vec![0], ())),
				|()| TARGET_VERSION.put::<CommunitiesManager>(),
			),
		}
	}
}

impl SteppedMigration for MembershipsToGroupCollections {
	type Cursor = Cursor;
	type Identifier = [u8; 32];

	fn id() -> Self::Identifier {
		*b"kreivo/memberships-to-group-coll"
	}

	fn step(cursor: Option<Cursor>, meter: &mut WeightMeter) -> Result<Option<Cursor>, SteppedMigrationError> {
		let mut cursor = match cursor {
			Some(cursor) => cursor,
			None => {
				// The gate: a chain that already is on `communities-manager`'s version 1 has
				// nothing to migrate.
				if CommunitiesManager::on_chain_storage_version() >= TARGET_VERSION {
					return Ok(None);
				}
				Cursor::start_of(Phase::Guard)
			}
		};

		// Only a unit that doesn't fit an empty block is an error (a bug).
		let required = cursor.phase.unit();
		if meter.remaining().any_lt(required) {
			return Err(SteppedMigrationError::InsufficientWeight { required });
		}

		loop {
			let last = cursor.last.as_ref().map(|raw| raw.to_vec());
			match Self::run(cursor.phase, meter, last) {
				Outcome::Yield(last) => {
					cursor.last = last.map(BoundedVec::truncate_from);
					return Ok(Some(cursor));
				}
				Outcome::Done => match cursor.phase.next() {
					Some(phase) => cursor = Cursor::start_of(phase),
					None => {
						log::info!(target: LOG_TARGET, "memberships migration: done");
						return Ok(None);
					}
				},
			}
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
		Self::pre_upgrade_checks()
			.map(|snapshot| snapshot.encode())
			.map_err(Into::into)
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
		let snapshot = Snapshot::decode(&mut &state[..]).map_err(|_| "the snapshot doesn't decode")?;
		Self::post_upgrade_checks(&snapshot).map_err(Into::into)
	}
}

/// What `pre_upgrade` records for `post_upgrade` (PLAN §6.3's table).
#[derive(Clone, Debug, Default, PartialEq, Eq, Encode, Decode)]
pub struct Snapshot {
	/// Every member item: community, id, holder, rank.
	pub members: Vec<(CommunityId, MembershipId, AccountId, Option<u32>)>,
	/// The stock the communities hold in the store: community, id.
	pub stock: Vec<(CommunityId, MembershipId)>,
	/// The refund rows, with the holders found: item, holder, amount.
	pub refunds: Vec<(MembershipId, AccountId, Balance)>,
	/// Every registered community.
	pub communities: Vec<CommunityId>,
	/// Members and rank sum per collection.
	pub totals: Vec<(CommunityId, u32, u32)>,
	/// The highest membership id, in any collection.
	pub highest_id: Option<MembershipId>,
	/// The Treasury's free balance, and the holders' of the refunds.
	pub treasury_free: Balance,
	/// What the burns (M2, M4) and the attribute clears (M5) release to the Treasury: the item,
	/// metadata and attribute deposits it paid (or owes as the store's owner).
	pub released_to_treasury: Balance,
	/// The memberships (members and stock) that M6 will fund: those without a deposit row yet.
	pub unfunded: u32,
	pub holders_free: Vec<(AccountId, Balance)>,
	/// The communities that already record a storage part.
	pub with_storage_part: Vec<CommunityId>,
	/// The collections, for M8.
	pub collections: Vec<CommunityId>,
}

type CheckResult<T = ()> = Result<T, &'static str>;

fn ensure(condition: bool, error: &'static str) -> CheckResult {
	condition.then_some(()).ok_or(error)
}

fn has_tank_keys() -> bool {
	Attribute::<Runtime, I>::iter_keys().any(|(_, _, _, key)| TANK_KEYS.iter().any(|tank| key.ends_with(tank)))
}

fn rank_of(collection: CommunityId, item: MembershipId) -> Option<u32> {
	<crate::Memberships as Rank<AccountId>>::rank_of(&collection, &item).map(Into::into)
}

impl MembershipsToGroupCollections {
	/// The `pre_upgrade` rows of PLAN §6.3: they record what `post_upgrade` compares with, and fail
	/// the rehearsal on what the steps would only skip and log.
	pub fn pre_upgrade_checks() -> CheckResult<Snapshot> {
		let mut snapshot = Snapshot::default();

		// M0 (`REQ-RT-2`).
		ensure(
			!has_tank_keys(),
			"a membership still has a gas tank or expiration attribute",
		)?;

		// Every member item, and the ids' uniqueness across the collections (0009-A21).
		let mut seen = alloc::collections::BTreeSet::new();
		for (collection, item) in ItemOf::iter_keys() {
			snapshot.highest_id = snapshot.highest_id.max(Some(item));
			if collection == STORE {
				continue;
			}
			ensure(seen.insert(item), "a membership id is in more than one collection")?;
			let holder = owner_of(collection, item).ok_or("a member item has no holder")?;
			// `RISK-10`: nothing locked an item or a collection against transfers.
			ensure(
				ItemConfigOf::<Runtime, I>::get(collection, item)
					.is_none_or(|c| !c.has_disabled_setting(ItemSetting::Transferable)),
				"a member item has transfers disabled",
			)?;
			snapshot
				.members
				.push((collection, item, holder, rank_of(collection, item)));
		}

		// Each collection: transferable items (`RISK-10`), no deposit owed (M8).
		for collection in Collection::<Runtime, I>::iter_keys() {
			let config = CollectionConfigOf::<Runtime, I>::get(collection).ok_or("a collection has no config")?;
			ensure(
				config.is_setting_enabled(CollectionSetting::TransferableItems),
				"a collection has transfers disabled",
			)?;
			ensure(
				Collection::<Runtime, I>::get(collection).is_some_and(|details| details.owner_deposit.is_zero()),
				"a collection has an owner deposit",
			)?;
			snapshot.collections.push(collection);
			if collection != STORE {
				snapshot.totals.push((
					collection,
					crate::Memberships::members_total(&collection),
					crate::Memberships::ranks_total(&collection),
				));
			}
		}

		// The stock the communities hold in the store (M2).
		for item in ItemOf::iter_key_prefix(STORE) {
			if let Some(community) = owner_of(STORE, item).as_ref().and_then(community_of) {
				snapshot.stock.push((community, item));
			}
		}

		// The refund rows (0007-A2): each item in the store, held by its community's admin.
		let mut total_refunds: Balance = 0;
		for (item, community, amount) in REFUNDS {
			let holder = owner_of(STORE, item).ok_or("a refunded item is not in the store")?;
			ensure(
				is_admin_of(&holder, community),
				"a refunded item is not held by its community's admin",
			)?;
			total_refunds = total_refunds.saturating_add(amount);
			snapshot
				.holders_free
				.push((holder.clone(), Balances::free_balance(&holder)));
			snapshot.refunds.push((item, holder, amount));
		}

		// The Treasury covers M6, M9 and M11 together.
		snapshot.communities = Info::iter_keys().collect();
		snapshot.with_storage_part = CommunityDeposits::<Runtime>::iter_keys().collect();
		snapshot.treasury_free = Balances::free_balance(treasury());
		snapshot.released_to_treasury = Self::released_to_treasury();
		snapshot.unfunded = (snapshot
			.members
			.iter()
			.filter(|(c, i, ..)| !Deposits::<Runtime>::contains_key((c, i)))
			.count()
			+ snapshot
				.stock
				.iter()
				.filter(|(c, i)| !Deposits::<Runtime>::contains_key((c, i)))
				.count()) as u32;
		let owed = Self::owed(&snapshot, total_refunds);
		ensure(
			Balances::reducible_balance(&treasury(), Preservation::Preserve, Fortitude::Polite) >= owed,
			"the Treasury cannot cover the deposits, the refunds and the storage parts",
		)?;

		Ok(snapshot)
	}

	/// What the burns and the clears give back to the Treasury (`pallet_nfts` unreserves each
	/// deposit to `deposit.account`, or to the collection's owner for an owner attribute without one):
	/// the item and metadata deposits of every store item (all burnt by M2 and M4), and the deposits
	/// of every attribute left on a missing item (M5's orphans, the burnt store items' included).
	fn released_to_treasury() -> Balance {
		let treasury = treasury();
		let owner_of_collection = |c: CommunityId| Collection::<Runtime, I>::get(c).map(|d| d.owner);
		let mut released: Balance = 0;
		for (item, details) in ItemOf::iter_prefix(STORE) {
			if details.deposit.account == treasury {
				released = released.saturating_add(details.deposit.amount);
			}
			let unlocked = ItemConfigOf::<Runtime, I>::get(STORE, item)
				.is_none_or(|c| c.is_setting_enabled(ItemSetting::UnlockedMetadata));
			if let (true, Some(metadata)) = (unlocked, pallet_nfts::ItemMetadataOf::<Runtime, I>::get(STORE, item)) {
				let payer = metadata.deposit.account.or_else(|| owner_of_collection(STORE));
				if payer.as_ref() == Some(&treasury) {
					released = released.saturating_add(metadata.deposit.amount);
				}
			}
		}
		for ((collection, item, namespace, _), (_, deposit)) in Attribute::<Runtime, I>::iter() {
			let Some(item) = item else { continue };
			if collection != STORE && ItemOf::contains_key(collection, item) {
				continue;
			}
			let payer = match (deposit.account, namespace) {
				(Some(account), _) => Some(account),
				(None, AttributeNamespace::CollectionOwner) => owner_of_collection(collection),
				_ => None,
			};
			if payer.as_ref() == Some(&treasury) {
				released = released.saturating_add(deposit.amount);
			}
		}
		released
	}

	/// What the migration takes from the Treasury.
	fn owed(snapshot: &Snapshot, refunds: Balance) -> Balance {
		let memberships = snapshot.unfunded as u128;
		let parts = snapshot
			.communities
			.iter()
			.filter(|c| !snapshot.with_storage_part.contains(c))
			.count() as u128;
		memberships
			.saturating_mul(MembershipDepositAmount::get())
			.saturating_add(refunds)
			.saturating_add(parts.saturating_mul(CommunityStoragePartAmount::get()))
	}

	/// The `post_upgrade` rows of PLAN §6.3.
	pub fn post_upgrade_checks(before: &Snapshot) -> CheckResult {
		let locked = |collection: &CommunityId, item: &MembershipId| {
			!<CommunityMemberships as nfv2::Inspect<AccountId>>::can_transfer(collection, item)
		};

		// AC-G3.1: each member keeps its holder and rank, and is locked.
		for (collection, item, holder, rank) in &before.members {
			ensure(
				owner_of(*collection, *item).as_ref() == Some(holder),
				"a member's holder changed",
			)?;
			ensure(rank_of(*collection, *item) == *rank, "a member's rank changed")?;
			ensure(locked(collection, item), "a member is not locked")?;
		}
		// AC-G3.2: the stock is in its community's collection, held by its account, locked.
		for (community, item) in &before.stock {
			ensure(
				owner_of(*community, *item) == Some(Communities::community_account(community)),
				"a stock item is not held by its community",
			)?;
			ensure(locked(community, item), "a stock item is not locked")?;
		}
		// Refunds: the items are gone, and the holders were paid.
		let mut paid: Balance = 0;
		for (item, holder, amount) in &before.refunds {
			ensure(owner_of(STORE, *item).is_none(), "a refunded item still exists")?;
			ensure(*holder != treasury(), "a refund goes to the Treasury")?;
			paid = paid.saturating_add(*amount);
		}
		for (holder, free) in &before.holders_free {
			let due: Balance = before
				.refunds
				.iter()
				.filter(|(_, h, _)| h == holder)
				.map(|(_, _, a)| *a)
				.sum();
			ensure(
				Balances::free_balance(holder) == free.saturating_add(due),
				"a holder was not refunded exactly",
			)?;
		}
		ensure(PendingRefunds::iter().next().is_none(), "a refund is still pending")?;
		// REQ-RT-7, INV-19: the Treasury fell by exactly the total, less what the burns and the
		// clears released to it.
		let expected = before
			.treasury_free
			.saturating_sub(Self::owed(before, paid))
			.saturating_add(before.released_to_treasury);
		let actual = Balances::free_balance(treasury());
		if actual != expected {
			let memberships = before.unfunded as u128;
			let parts = before
				.communities
				.iter()
				.filter(|c| !before.with_storage_part.contains(c))
				.count() as u128;
			log::error!(
				target: LOG_TARGET,
				"memberships migration: the Treasury was {} before, is {actual} after, and should be {expected} \
				 (M6 deposits {} for {memberships} memberships, M9 refunds {paid}, M11 storage parts {} for \
				 {parts} communities, nft deposits released to the Treasury {})",
				before.treasury_free,
				memberships.saturating_mul(MembershipDepositAmount::get()),
				parts.saturating_mul(CommunityStoragePartAmount::get()),
				before.released_to_treasury,
			);
			return Err("the Treasury didn't fall by exactly the total");
		}

		// Totals unchanged.
		for (collection, members, ranks) in &before.totals {
			ensure(
				crate::Memberships::members_total(collection) == *members
					&& crate::Memberships::ranks_total(collection) == *ranks,
				"a collection's totals changed",
			)?;
		}

		// AC-G3.3: the store is empty and leaves nothing behind.
		ensure(
			Collection::<Runtime, I>::get(STORE).is_some_and(|d| d.items == 0),
			"the store still counts items",
		)?;
		ensure(
			ItemOf::iter_key_prefix(STORE).next().is_none(),
			"the store still has items",
		)?;
		ensure(
			ItemPriceOf::<Runtime, I>::iter_key_prefix(STORE).next().is_none(),
			"a store price remains",
		)?;
		ensure(
			ItemConfigOf::<Runtime, I>::iter_key_prefix(STORE).next().is_none(),
			"a store item config remains",
		)?;
		ensure(
			pallet_nfts::Account::<Runtime, I>::iter_keys().all(|(_, collection, _)| collection != STORE),
			"an account still holds a store item",
		)?;
		// REQ-RT-6: no attribute on a missing item, in any collection.
		ensure(
			Attribute::<Runtime, I>::iter_keys()
				.all(|(c, item, _, _)| item.is_none_or(|item| ItemOf::contains_key(c, item))),
			"an attribute is left on a missing item",
		)?;
		ensure(!has_tank_keys(), "a gas tank or expiration attribute appeared")?;

		// Ids.
		if let Some(highest) = before.highest_id {
			ensure(
				NextMembershipId::<Runtime>::get() == highest.saturating_add(1),
				"NextMembershipId is not the highest id plus one",
			)?;
		}

		// Deposits (M6), the index (M10).
		let deposit = MembershipDepositAmount::get();
		let mut per_community = alloc::collections::BTreeMap::<CommunityId, u128>::new();
		for (collection, item) in ItemOf::iter_keys().filter(|(c, _)| *c != STORE) {
			ensure(
				Deposits::<Runtime>::get((collection, item)) == Some(deposit),
				"a membership has no deposit",
			)?;
			*per_community.entry(collection).or_default() += deposit;
			ensure(
				frame_support::storage::unhashed::get::<CommunityId>(&membership_index_key(&item)) == Some(collection),
				"a membership is not indexed under its group",
			)?;
		}
		for (collection, held) in per_community {
			ensure(
				Balances::balance_on_hold(
					&hold_reason(HoldReason::MembershipDeposit),
					&Communities::community_account(&collection),
				) == held,
				"a community doesn't hold its memberships' deposits",
			)?;
		}

		// Storage parts (M11).
		let part = CommunityStoragePartAmount::get();
		for community in &before.communities {
			if before.with_storage_part.contains(community) || part.is_zero() {
				continue;
			}
			ensure(
				CommunityDeposits::<Runtime>::get(community) == Some(part)
					&& Balances::balance_on_hold(
						&hold_reason(HoldReason::CommunityDeposit),
						&Communities::community_account(community),
					) >= part,
				"a community doesn't hold its storage part",
			)?;
		}

		// M8: every collection belongs to the manager, which holds every role alone.
		let manager = MembershipsManagerAccount::get();
		for collection in &before.collections {
			ensure(
				Collection::<Runtime, I>::get(collection)
					.is_some_and(|d| d.owner == manager && d.owner_deposit.is_zero()),
				"a collection is not owned by the manager",
			)?;
			let roles: Vec<_> = CollectionRoleOf::<Runtime, I>::iter_prefix(collection).collect();
			ensure(
				roles.len() == 1 && roles[0].0 == manager && roles[0].1 .0.bits() == 7,
				"a collection's team is not the manager alone",
			)?;
		}
		ensure(
			CollectionAccount::<Runtime, I>::iter_keys().all(|(owner, _)| owner == manager),
			"a collection is listed under another owner",
		)?;

		ensure(
			CommunitiesManager::on_chain_storage_version() == TARGET_VERSION,
			"the storage version is not 1",
		)?;
		Ok(())
	}
}
