//! `MembershipsToGroupCollections`, the stepped migration of the memberships (`F-10`, PLAN §6.3).
//!
//! The state mirrors live's shapes (PLAN §1.5): a store (collection 0) with priced stock, twins held
//! by the assigned-memberships account, stock held by community accounts, the 7 admin-held items,
//! and members in the communities' collections, owned by their community accounts. The values are
//! test values: nothing here hard-codes the deposit or the storage part.

use super::*;

use crate::{
	config::{
		communities::{CommunityStoragePartAmount, MembershipDepositAmount, MembershipsManagerAccount},
		system::MigrationsMaxServiceWeight,
	},
	migrations::memberships::*,
	CommunityMemberships, RuntimeHoldReason,
};
use frame_contrib_traits::memberships::{
	GenericRank, Inspect as _, Rank, ATTR_MEMBER_RANK, ATTR_MEMBER_RANK_TOTAL, ATTR_MEMBER_TOTAL,
};
use frame_support::{
	migrations::{SteppedMigration, SteppedMigrationError},
	traits::{
		fungible::hold::Inspect as _,
		nonfungibles_v2::{self as nfv2, Create},
		Get,
	},
	weights::{Weight, WeightMeter},
};
use pallet_communities_manager::{CommunityDeposits, Deposits, HoldReason, NextMembershipId};
use pallet_nfts::{Item, ItemConfig, ItemPriceOf};

type Migration = MembershipsToGroupCollections;
type Instance = crate::config::communities::memberships::CommunityMembershipsInstance;

const ALICE: AccountId32 = AccountId32::new([1; 32]);
const BOB: AccountId32 = AccountId32::new([2; 32]);
const CAROL: AccountId32 = AccountId32::new([3; 32]);
const DAVE: AccountId32 = AccountId32::new([4; 32]);
const EVE: AccountId32 = AccountId32::new([5; 32]);
const FRANK: AccountId32 = AccountId32::new([6; 32]);
const GRACE: AccountId32 = AccountId32::new([7; 32]);
const HEIDI: AccountId32 = AccountId32::new([8; 32]);
const STRANGER: AccountId32 = AccountId32::new([9; 32]);

fn treasury() -> AccountId {
	crate::config::TreasuryAccount::get()
}

/// The account that held the twins of assigned memberships.
fn assigned() -> AccountId {
	sp_runtime::str_array("memberships/assigned_memberships").into()
}

fn account(community: u16) -> AccountId {
	Communities::community_account(&community)
}

fn mint(collection: u16, item: u32, owner: &AccountId) {
	use frame_support::traits::ReservableCurrency;

	assert_ok!(<CommunityMemberships as nfv2::Mutate<_, _>>::mint_into(
		&collection,
		&item,
		owner,
		&ItemConfig::default(),
		false,
	));
	// Live's items have no deposit; the benchmark build gives them one, which would make every
	// balance a test compares drift.
	Item::<Runtime, Instance>::mutate(collection, item, |details| {
		if let Some(details) = details {
			Balances::unreserve(&details.deposit.account, details.deposit.amount);
			details.deposit.amount = 0;
		}
	});
}

fn exists(collection: u16, item: u32) -> bool {
	Item::<Runtime, Instance>::contains_key(collection, item)
}

fn holder(collection: u16, item: u32) -> Option<AccountId> {
	<CommunityMemberships as nfv2::Inspect<AccountId>>::owner(&collection, &item)
}

fn locked(collection: u16, item: u32) -> bool {
	!<CommunityMemberships as nfv2::Inspect<AccountId>>::can_transfer(&collection, &item)
}

fn indexed(item: u32) -> Option<u16> {
	frame_support::storage::unhashed::get(&membership_index_key(&item))
}

fn free(who: &AccountId) -> u128 {
	Balances::free_balance(who)
}

fn held(community: u16, reason: HoldReason) -> u128 {
	Balances::balance_on_hold(&RuntimeHoldReason::CommunitiesManager(reason), &account(community))
}

fn register(community: u16, admin: AccountId32) {
	assert_ok!(CommunitiesManager::register(
		RuntimeOrigin::root(),
		community,
		BoundedVec::try_from(b"A Community".to_vec()).expect("meets max length; qed"),
		CommunityLookup::unlookup(admin),
		None,
		None,
	));
	// Live's collections belong to their community accounts, which hold every role.
	let owner = CommunityLookup::unlookup(account(community));
	assert_ok!(pallet_nfts::Pallet::<Runtime, Instance>::force_collection_owner(
		RuntimeOrigin::root(),
		community,
		owner.clone(),
	));
	assert_ok!(pallet_nfts::Pallet::<Runtime, Instance>::set_team(
		RuntimeOrigin::root(),
		community,
		Some(owner.clone()),
		Some(owner.clone()),
		Some(owner),
	));
}

/// Marks `item` of `collection` as a member's: its rank, and the collection's totals.
fn member(collection: u16, item: u32, holder: &AccountId, rank: u8, total: u32, rank_total: u32) {
	mint(collection, item, holder);
	assert_ok!(<CommunityMemberships as nfv2::Mutate<_, _>>::set_typed_attribute(
		&collection,
		&item,
		&ATTR_MEMBER_RANK,
		&GenericRank::from(rank),
	));
	assert_ok!(
		<CommunityMemberships as nfv2::Mutate<_, _>>::set_typed_collection_attribute(
			&collection,
			&ATTR_MEMBER_TOTAL,
			&total,
		)
	);
	assert_ok!(
		<CommunityMemberships as nfv2::Mutate<_, _>>::set_typed_collection_attribute(
			&collection,
			&ATTR_MEMBER_RANK_TOTAL,
			&rank_total,
		)
	);
}

/// Live's shapes, in small: communities 1, 69, 777, 1107 and 1786; the store; members, twins,
/// stock and the 7 admin-held items.
fn live_state() {
	for who in [ALICE, BOB, CAROL, DAVE, EVE, FRANK, GRACE, HEIDI, STRANGER] {
		assert_ok!(Balances::mint_into(&who, UNITS));
	}
	assert_ok!(Balances::mint_into(&treasury(), 1_000 * UNITS));
	assert_ok!(Balances::mint_into(&assigned(), UNITS));
	assert_ok!(Balances::mint_into(&MembershipsManagerAccount::get(), UNITS));

	// The store, owned by the Treasury.
	assert_ok!(<CommunityMemberships as Create<_, _>>::create_collection_with_id(
		0,
		&treasury(),
		&treasury(),
		&Default::default(),
	));
	for (community, admin) in [(1, ALICE), (69, BOB), (777, CAROL), (1107, DAVE), (1786, EVE)] {
		register(community, admin);
		assert_ok!(Balances::mint_into(&account(community), UNITS));
	}

	// Priced stock, held by the Treasury; one item carries an attribute.
	for item in 1..=4 {
		mint(0, item, &treasury());
		ItemPriceOf::<Runtime, Instance>::insert(0u16, item, (3 * UNITS / 10, None::<AccountId>));
	}
	assert_ok!(<CommunityMemberships as nfv2::Mutate<_, _>>::set_typed_attribute(
		&0,
		&4,
		&b"a_leftover",
		&1u8,
	));

	// Members, in their communities' collections, each with a twin in the store.
	member(1, 200, &FRANK, 3, 1, 3);
	member(1786, 201, &GRACE, 5, 2, 12);
	member(1786, 202, &HEIDI, 7, 2, 12);
	for item in [200, 201, 202] {
		mint(0, item, &assigned());
	}

	// Stock the communities hold in the store.
	mint(0, 300, &account(1));
	mint(0, 301, &account(1));
	mint(0, 302, &account(1786));

	// The 7 admin-held items.
	for (item, community, _) in REFUNDS {
		let admin = match community {
			69 => BOB,
			777 => CAROL,
			1_107 => DAVE,
			_ => EVE,
		};
		mint(0, item, &admin);
	}
}

fn state_root() -> Vec<u8> {
	sp_io::storage::root(sp_runtime::StateVersion::V1)
}

fn run_phase(phase: Phase) {
	let mut meter = WeightMeter::with_limit(Weight::MAX);
	assert!(matches!(
		Migration::run(phase, &mut meter, None),
		crate::migrations::memberships::Outcome::Done
	));
}

/// Runs every phase up to, and with, `last`, each to its end.
fn through(last: Phase) {
	let mut phase = Some(Phase::Guard);
	while let Some(current) = phase {
		run_phase(current);
		if current == last {
			return;
		}
		phase = current.next();
	}
}

/// Steps the migration to its end with `limit` a step, and returns the steps taken.
fn run_to_the_end(limit: Weight) -> u32 {
	let mut cursor = None;
	let mut steps = 0;
	loop {
		let mut meter = WeightMeter::with_limit(limit);
		cursor = Migration::step(cursor, &mut meter).expect("a step passes");
		assert!(meter.consumed().all_lte(limit));
		steps += 1;
		if cursor.is_none() {
			return steps;
		}
		assert!(steps < 10_000, "the migration doesn't end");
	}
}

fn max_service() -> Weight {
	MigrationsMaxServiceWeight::get()
}

// T-10-01, NFR-9: on an empty state the migration completes in one step and sets version 1; a
// second run changes nothing.
#[test]
fn an_empty_state_completes_in_one_step_and_sets_version_1() {
	use frame_support::traits::GetStorageVersion;

	TestExternalities::default().execute_with(|| {
		assert_eq!(CommunitiesManager::on_chain_storage_version(), 0);
		assert_eq!(run_to_the_end(max_service()), 1);
		assert_eq!(CommunitiesManager::on_chain_storage_version(), 1);

		let root = state_root();
		let mut meter = WeightMeter::with_limit(max_service());
		assert_eq!(Migration::step(None, &mut meter), Ok(None));
		assert_eq!(meter.consumed(), Weight::zero());
		assert_eq!(state_root(), root);
	});
}

// T-10-02, REQ-RT-5, DEC-20, AC-G1.2: the counter ends above every id that ever existed, the store's
// included.
#[test]
fn the_counter_is_above_every_id() {
	TestExternalities::default().execute_with(|| {
		live_state();
		through(Phase::Ids);
		assert_eq!(NextMembershipId::<Runtime>::get(), 10_442);
	});
}

// T-10-03, REQ-RT-4, AC-G3.2: the stock a community held in the store moves into its collection,
// held by its account, locked, with the same id.
#[test]
fn stock_held_by_a_community_moves_into_its_collection() {
	TestExternalities::default().execute_with(|| {
		live_state();
		through(Phase::Stock);
		for (community, item) in [(1, 300), (1, 301), (1786, 302)] {
			assert!(!exists(0, item));
			assert_eq!(holder(community, item), Some(account(community)));
			assert!(locked(community, item));
		}
		// Not stock: the Treasury's, the twins and the admins' are left for the store's retirement.
		assert!(exists(0, 1) && exists(0, 200) && exists(0, 100));
	});
}

// T-10-03, REQ-RT-4, AC-G3.1: members are locked; holder, rank and totals are unchanged.
#[test]
fn members_are_locked_and_keep_their_holder_rank_and_totals() {
	TestExternalities::default().execute_with(|| {
		live_state();
		assert!(!locked(1, 200));
		through(Phase::Lock);
		for (community, item, who, rank) in [(1, 200, FRANK, 3), (1786, 201, GRACE, 5), (1786, 202, HEIDI, 7)] {
			assert!(locked(community, item));
			assert_eq!(holder(community, item), Some(who));
			assert_eq!(
				<crate::Memberships as Rank<AccountId>>::rank_of(&community, &item),
				Some(GenericRank::from(rank))
			);
		}
		assert_eq!(crate::Memberships::members_total(&1786), 2);
		assert_eq!(<crate::Memberships as Rank<AccountId>>::ranks_total(&1786), 12);
	});
}

// T-10-03, REQ-RT-6, DEC-33, AC-G3.3: the store ends empty, with no price, config or account entry,
// and the attribute a burnt item left is cleared as an orphan. M4 records the refunds M9 pays.
#[test]
fn the_store_is_retired_and_its_orphans_cleared() {
	TestExternalities::default().execute_with(|| {
		live_state();
		through(Phase::Stock);
		through(Phase::Retire);
		assert_eq!(Item::<Runtime, Instance>::iter_key_prefix(0).count(), 0);
		assert_eq!(ItemPriceOf::<Runtime, Instance>::iter_key_prefix(0).count(), 0);
		assert_eq!(PendingRefunds::iter().count(), 7);
		// The community's collection and the members stay.
		assert!(exists(1, 300) && exists(1, 200));

		let before = pallet_nfts::Attribute::<Runtime, Instance>::iter_keys().count();
		through(Phase::Orphans);
		assert_eq!(
			pallet_nfts::Attribute::<Runtime, Instance>::iter_keys().count(),
			before - 1
		);
		assert!(pallet_nfts::Attribute::<Runtime, Instance>::iter_keys()
			.all(|(c, item, _, _)| item.is_none_or(|item| exists(c, item))));
	});
}

// T-10-04, REQ-RT-7, INV-19, DEC-33: every membership holds a deposit on its community's account,
// recorded, and the Treasury falls by exactly the total.
#[test]
fn every_membership_holds_a_deposit_from_the_treasury() {
	TestExternalities::default().execute_with(|| {
		live_state();
		through(Phase::Retire);
		let (before, deposit) = (free(&treasury()), MembershipDepositAmount::get());
		through(Phase::Deposits);

		// 3 members and 3 stock items.
		assert_eq!(free(&treasury()), before - 6 * deposit);
		assert_eq!(held(1, HoldReason::MembershipDeposit), 3 * deposit);
		assert_eq!(held(1786, HoldReason::MembershipDeposit), 3 * deposit);
		for (community, item) in [(1, 200), (1, 300), (1, 301), (1786, 201), (1786, 202), (1786, 302)] {
			assert_eq!(Deposits::<Runtime>::get((community, item)), Some(deposit));
		}
		assert_eq!(Deposits::<Runtime>::iter().count(), 6);
	});
}

// T-10-05, DEC-36, 0004-A1: every collection is owned by the manager, which alone holds every role.
#[test]
fn the_manager_owns_every_collection() {
	TestExternalities::default().execute_with(|| {
		live_state();
		through(Phase::Manager);
		let manager = MembershipsManagerAccount::get();
		for collection in [0, 1, 69, 777, 1107, 1786] {
			assert_eq!(
				pallet_nfts::Collection::<Runtime, Instance>::get(collection).map(|d| d.owner),
				Some(manager.clone())
			);
			let roles: Vec<_> = pallet_nfts::CollectionRoleOf::<Runtime, Instance>::iter_prefix(collection).collect();
			assert_eq!(roles.len(), 1);
			assert_eq!(roles[0].0, manager);
		}
		assert!(pallet_nfts::CollectionAccount::<Runtime, Instance>::iter_keys().all(|(owner, _)| owner == manager));
	});
}

// T-10-04, DEP-18, 0007-A2: each admin is refunded what its item cost, and the Treasury pays 2.8 KSM.
#[test]
fn the_admins_of_the_seven_items_are_refunded() {
	TestExternalities::default().execute_with(|| {
		live_state();
		let admins = [CAROL, DAVE, BOB, EVE];
		through(Phase::Manager);
		let admins_before: Vec<_> = admins.iter().map(free).collect();
		let treasury_before = free(&treasury());
		run_phase(Phase::Refunds);

		assert_eq!(free(&treasury()), treasury_before - 28 * UNITS / 10);
		// 777: two items at 0.3; 1107: item 100 at 1.0; 69: 0.3; 1786: three at 0.3.
		for (admin, (before, due)) in
			admins.iter().zip(
				admins_before
					.into_iter()
					.zip([6 * UNITS / 10, UNITS, 3 * UNITS / 10, 9 * UNITS / 10]),
			) {
			assert_eq!(free(admin), before + due);
		}
		assert_eq!(PendingRefunds::iter().count(), 0);
	});
}

// T-10-05, 0009-A19.3, INV-17: every membership is indexed under its group.
#[test]
fn every_membership_is_indexed() {
	TestExternalities::default().execute_with(|| {
		live_state();
		through(Phase::Retire);
		through(Phase::Index);
		for (community, item) in [(1, 200), (1, 300), (1, 301), (1786, 201), (1786, 202), (1786, 302)] {
			assert_eq!(indexed(item), Some(community));
		}
		assert_eq!(indexed(1), None);
	});
}

// T-10-04, DEC-38, 0010-Q7: each registered community holds its storage part, recorded.
#[test]
fn every_community_holds_its_storage_part() {
	TestExternalities::default().execute_with(|| {
		live_state();
		through(Phase::Index);
		let (before, part) = (free(&treasury()), CommunityStoragePartAmount::get());
		run_phase(Phase::StorageParts);

		assert_eq!(free(&treasury()), before - 5 * part);
		for community in [1, 69, 777, 1107, 1786] {
			assert_eq!(CommunityDeposits::<Runtime>::get(community), Some(part));
			assert_eq!(held(community, HoldReason::CommunityDeposit), part);
		}
	});
}

// T-10-00, T-10-07: the whole migration, in one step, then again: nothing changes, and the checks of
// PLAN §6.3 hold before and after.
#[test]
fn the_migration_passes_the_checks_and_a_second_run_changes_nothing() {
	TestExternalities::default().execute_with(|| {
		live_state();
		let snapshot = Migration::pre_upgrade_checks().expect("the state is as the rehearsal needs");
		assert_eq!(run_to_the_end(max_service()), 1);
		assert_eq!(Migration::post_upgrade_checks(&snapshot), Ok(()));

		let root = state_root();
		assert_eq!(run_to_the_end(max_service()), 1);
		assert_eq!(state_root(), root);
	});
}

// T-10-00: a phase that spans more items than a block's weight yields, and the next block resumes
// at the cursor: the store's items go three at a time, in storage order.
#[test]
fn a_phase_spanning_more_items_than_one_block_yields_and_resumes_at_its_cursor() {
	TestExternalities::default().execute_with(|| {
		live_state();
		through(Phase::Stock);
		let order: Vec<u32> = Item::<Runtime, Instance>::iter_key_prefix(0).collect();
		assert!(order.len() > 6);

		let unit = Phase::Retire.unit();
		let limit = unit.saturating_mul(3);
		let mut cursor = Some(Cursor::start_of(Phase::Retire));
		let mut remaining = order.len();
		while remaining > 0 {
			let mut meter = WeightMeter::with_limit(limit);
			cursor = Migration::step(cursor, &mut meter).expect("a step passes");
			let burnt = remaining.min(3);
			remaining -= burnt;
			if remaining == 0 {
				break;
			}
			// Three items burnt, in order, and the cursor stands at the third.
			let cursor = cursor.clone().expect("it yields");
			assert_eq!(cursor.phase, Phase::Retire);
			let done = order.len() - remaining;
			assert_eq!(
				cursor.last.map(|key| key.to_vec()),
				Some(Item::<Runtime, Instance>::hashed_key_for(0, order[done - 1]))
			);
			assert_eq!(Item::<Runtime, Instance>::iter_key_prefix(0).count(), remaining);
			assert!(order[..done].iter().all(|item| !exists(0, *item)));
		}
		assert_eq!(Item::<Runtime, Instance>::iter_key_prefix(0).count(), 0);
	});
}

// T-10-07, NFR-9, AC-G3.4: a step a unit long, interrupting every phase boundary and mid-phase, ends
// in the state a single step reaches.
#[test]
fn interrupting_at_every_unit_ends_in_the_same_state() {
	let single = TestExternalities::default().execute_with(|| {
		live_state();
		run_to_the_end(max_service());
		state_root()
	});
	TestExternalities::default().execute_with(|| {
		live_state();
		let steps = run_to_the_end(heaviest_unit());
		assert!(steps > 6, "it takes many steps");
		assert_eq!(state_root(), single);
	});
}

// T-10-00: a step whose meter doesn't hold one unit of an empty block is the only error.
#[test]
fn a_meter_that_cannot_hold_one_unit_is_the_only_error() {
	TestExternalities::default().execute_with(|| {
		let mut meter = WeightMeter::with_limit(Phase::Guard.unit().saturating_sub(Weight::from_parts(1, 0)));
		assert_eq!(
			Migration::step(None, &mut meter),
			Err(SteppedMigrationError::InsufficientWeight {
				required: Phase::Guard.unit()
			})
		);
	});
}

// T-10-00, NFR-9: the heaviest unit fits `MaxServiceWeight`, with room for ten of it, in both
// components.
#[test]
fn the_heaviest_unit_fits_the_max_service_weight() {
	assert!(heaviest_unit().saturating_mul(10).all_lte(max_service()));
	let mut phase = Some(Phase::Guard);
	while let Some(current) = phase {
		assert!(current.unit().all_lte(heaviest_unit()));
		phase = current.next();
	}
	// A cursor fits the pallet's.
	let longest = Cursor {
		phase: Phase::Version,
		last: Some(sp_runtime::BoundedVec::truncate_from(vec![0u8; 256])),
	};
	assert!(
		parity_scale_codec::Encode::encoded_size(&longest)
			<= <<Runtime as pallet_migrations::Config>::CursorMaxLen as Get<u32>>::get() as usize
	);
}

// T-10-00, 0009-A21: a duplicate id is skipped and logged, never an error; the others are indexed.
#[test]
fn a_duplicate_id_is_skipped_when_indexing() {
	TestExternalities::default().execute_with(|| {
		live_state();
		// Item 200 was already indexed as another group's.
		frame_support::storage::unhashed::put(&membership_index_key(&200), &1786u16);
		assert_eq!(run_to_the_end(max_service()), 1);
		assert_eq!(indexed(200), Some(1786));
		assert_eq!(indexed(201), Some(1786));
		assert_eq!(indexed(300), Some(1));
	});
}

// T-10-00: a stock item whose id a member already has in the community's collection stays in the
// store and is burnt with it; nothing is lost but that item, and the step passes.
#[test]
fn a_stock_item_that_cannot_move_is_skipped() {
	TestExternalities::default().execute_with(|| {
		live_state();
		mint(0, 400, &account(1));
		member(1, 400, &FRANK, 1, 2, 4);
		through(Phase::Stock);
		assert!(exists(0, 400));
		assert_eq!(holder(1, 400), Some(FRANK));
		assert_eq!(run_to_the_end(max_service()), 1);
		assert!(!exists(0, 400));
		assert_eq!(holder(1, 400), Some(FRANK));
	});
}

// T-10-00, DEP-18, 0007-A2: a refund row that no longer matches is skipped: its holder is paid
// nothing, and the others are paid.
#[test]
fn a_refund_row_that_no_longer_matches_is_skipped() {
	TestExternalities::default().execute_with(|| {
		live_state();
		// Item 100 moved to an account that isn't the admin of 1107.
		assert_ok!(<CommunityMemberships as nfv2::Transfer<_>>::transfer(
			&0, &100, &STRANGER
		));
		let (stranger, dave, treasury_before) = (free(&STRANGER), free(&DAVE), free(&treasury()));
		assert_eq!(run_to_the_end(max_service()), 1);
		assert_eq!(free(&STRANGER), stranger);
		assert_eq!(free(&DAVE), dave);
		assert!(treasury_before - free(&treasury()) >= 18 * UNITS / 10);
		assert!(!exists(0, 100));
	});
}

// T-10-00, REQ-RT-7: a Treasury that cannot pay is skipped and logged, never an error: the migration
// still completes, and holds what it could. `pre_upgrade` is what fails the rehearsal.
#[test]
fn a_treasury_that_cannot_pay_is_skipped() {
	TestExternalities::default().execute_with(|| {
		live_state();
		Balances::set_balance(&treasury(), EXISTENTIAL_DEPOSIT);
		let snapshot_error = Migration::pre_upgrade_checks().err();
		assert!(snapshot_error.is_some());
		assert_eq!(run_to_the_end(max_service()), 1);
		assert_eq!(Deposits::<Runtime>::iter().count(), 0);
		assert_eq!(CommunityDeposits::<Runtime>::iter().count(), 0);
		assert_eq!(free(&treasury()), EXISTENTIAL_DEPOSIT);
	});
}

// T-10-00, REQ-RT-2: a tank attribute that survives is logged, not raised, and `pre_upgrade` fails.
#[test]
fn a_tank_attribute_is_skipped_by_the_steps_and_fails_pre_upgrade() {
	TestExternalities::default().execute_with(|| {
		live_state();
		assert_ok!(<CommunityMemberships as nfv2::Mutate<_, _>>::set_typed_attribute(
			&1,
			&200,
			&b"membership_gas",
			&1u8,
		));
		assert_eq!(
			Migration::pre_upgrade_checks().err(),
			Some("a membership still has a gas tank or expiration attribute")
		);
		assert_eq!(run_to_the_end(max_service()), 1);
	});
}

// T-10-06, 0009-A21: `pre_upgrade` fails on a membership id in two collections.
#[test]
fn pre_upgrade_fails_on_a_duplicate_id() {
	TestExternalities::default().execute_with(|| {
		live_state();
		mint(69, 200, &FRANK);
		assert_eq!(
			Migration::pre_upgrade_checks().err(),
			Some("a membership id is in more than one collection")
		);
	});
}

// T-10-06, 0007-A2: `pre_upgrade` fails on a refund row whose holder changed.
#[test]
fn pre_upgrade_fails_on_a_changed_holder() {
	TestExternalities::default().execute_with(|| {
		live_state();
		assert_ok!(<CommunityMemberships as nfv2::Transfer<_>>::transfer(
			&0, &100, &STRANGER
		));
		assert_eq!(
			Migration::pre_upgrade_checks().err(),
			Some("a refunded item is not held by its community's admin")
		);
	});
}

// T-10-06, RISK-10: `pre_upgrade` fails on a member item with transfers disabled by someone else.
#[test]
fn pre_upgrade_fails_on_a_locked_item() {
	TestExternalities::default().execute_with(|| {
		live_state();
		assert_ok!(pallet_nfts::Pallet::<Runtime, Instance>::lock_item_transfer(
			RuntimeOrigin::signed(account(1)),
			1,
			200,
		));
		assert_eq!(
			Migration::pre_upgrade_checks().err(),
			Some("a member item has transfers disabled")
		);
	});
}

// T-10-06, AC-G3.1: `post_upgrade` fails when a member is left unlocked.
#[test]
fn post_upgrade_fails_on_an_unlocked_member() {
	TestExternalities::default().execute_with(|| {
		live_state();
		let snapshot = Migration::pre_upgrade_checks().expect("the state is as the rehearsal needs");
		run_to_the_end(max_service());
		assert_ok!(<CommunityMemberships as nfv2::Transfer<_>>::enable_transfer(&1, &200));
		assert_eq!(Migration::post_upgrade_checks(&snapshot), Err("a member is not locked"));
	});
}

// T-10-06: the try-runtime hooks round-trip the snapshot.
#[cfg(feature = "try-runtime")]
#[test]
fn the_try_runtime_hooks_round_trip() {
	TestExternalities::default().execute_with(|| {
		live_state();
		let state = <Migration as SteppedMigration>::pre_upgrade().expect("pre_upgrade passes");
		run_to_the_end(max_service());
		assert_eq!(<Migration as SteppedMigration>::post_upgrade(state), Ok(()));
	});
}
