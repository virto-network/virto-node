//! The measured values of M-3's deposits, delay and limits (`F-07b`, `deployment/kreivo-0.18.md`).
//!
//! Each footprint is the sum, over the storage entries a record creates, of the `MaxEncodedLen` of
//! the entry's key and value (unhashed, as `Pallet::subscription_footprint()` counts them), and one
//! item per entry. The runtime binds each deposit as `deposit(items, bytes)`, Kreivo's existing
//! rate, so a change to a record that changes its footprint fails here.

use super::*;
use crate::{
	config::{
		communities::{
			memberships::CommunityMembershipsInstance, CommunityConsumptionPartAmount, CommunityDepositAmount,
			CommunityStoragePartAmount, MaxIssuePerCall, MembershipDepositAmount, RetirementDelayInBlocks,
			COMMUNITY_BYTES, COMMUNITY_ITEMS,
		},
		listings_orders::{
			ListingsDueBucketSize, ListingsMaxChargesPerBlock, ListingsMaxDuePerBucket, ListingsRenewalLead,
			SubscriptionDepositAmount, SUBSCRIPTION_BYTES,
		},
		RuntimeBlockWeights,
	},
	constants::currency::deposit,
	OriginCaller,
};
use frame_support::{dispatch::DispatchClass, weights::Weight};
use frame_system::AccountInfo;
use pallet_communities_manager::WeightInfo as _;
use pallet_listings::WeightInfo as _;
use parity_scale_codec::MaxEncodedLen;
use runtime_constants::time::{DAYS, HOURS, MINUTES, WEEKS};
use virto_common::{CommunityId, MembershipId};

/// Kreivo's deposit rate (`constants::currency::deposit`: 2 000 cents per item and 100 millicents
/// per byte, a hundredth of the relay chain's), written out: CENTS = 333 333 333 and
/// MILLICENTS = 333 333 planck.
///
/// Paseo's unit is a hundredth of Kusama's, so its constants are too: the literals below are
/// Kusama's (the deployed chain's), and Paseo is checked by formula only.
const PER_ITEM: u128 = 6_666_666_660;
const PER_BYTE: u128 = 333_333;
const KUSAMA: bool = !cfg!(feature = "paseo");

fn mel<T: MaxEncodedLen>() -> u32 {
	T::max_encoded_len() as u32
}

/// Items and bytes of a set of entries, each given as (key, value) maximum encoded lengths.
fn footprint(entries: &[(u32, u32)]) -> (u32, u32) {
	(entries.len() as u32, entries.iter().map(|(k, v)| k + v).sum())
}

type Account = <Runtime as frame_system::Config>::AccountId;

/// What registering a community writes (`Communities::register`, the collection, the track and the
/// collective's induction), per entry. `CommunityTracks::TracksIds` is one shared vector that grows
/// by one track id: two bytes, no item.
fn community_footprint() -> (u32, u32) {
	use pallet_ranked_collective::MemberRecord;
	type Info = pallet_communities::types::CommunityInfo;
	let (account, id, rank, index) = (mel::<Account>(), mel::<CommunityId>(), mel::<u16>(), mel::<u32>());
	let (items, bytes) = footprint(&[
		// System::Account (the community account, given a provider)
		(
			account,
			mel::<AccountInfo<<Runtime as frame_system::Config>::Nonce, <Runtime as frame_system::Config>::AccountData>>(
			),
		),
		// Communities::CommunityIdFor (the admin's origin → id)
		(mel::<OriginCaller>(), id),
		// Communities::Info
		(id, mel::<Info>()),
		// CommunityMemberships: Collection, CollectionRoleOf (the manager), CollectionConfigOf,
		// CollectionAccount
		(
			id,
			mel::<pallet_nfts::CollectionDetailsFor<Runtime, CommunityMembershipsInstance>>(),
		),
		(id + account, mel::<pallet_nfts::CollectionRoles>()),
		(
			id,
			mel::<pallet_nfts::CollectionConfigFor<Runtime, CommunityMembershipsInstance>>(),
		),
		(account + id, 0),
		// CommunityTracks: Tracks, OriginToTrackId
		(
			id,
			mel::<
				pallet_referenda::TrackInfoOf<
					Runtime,
					crate::config::communities::governance::CommunityReferendaInstance,
				>,
			>(),
		),
		(mel::<OriginCaller>(), id),
		// KreivoCollective (inducted at rank 0): Members, MemberCount, IdToIndex, IndexToId
		(account, mel::<MemberRecord>()),
		(rank, index),
		(rank + account, index),
		(rank + index, account),
	]);
	(items, bytes + mel::<CommunityId>())
}

/// What issuing a membership writes: the manager's deposit record, the nfts item, its config, its
/// owner index and the membership attribute, and the membership-to-community index.
fn membership_footprint() -> (u32, u32) {
	type I = CommunityMembershipsInstance;
	let (account, id, item) = (mel::<Account>(), mel::<CommunityId>(), mel::<MembershipId>());
	footprint(&[
		// CommunitiesManager::Deposits
		(id + item, mel::<u128>()),
		// membership-to-community index
		(item, id),
		// CommunityMemberships::Item, ItemConfigOf, Account, Attribute
		(id + item, mel::<pallet_nfts::ItemDetailsFor<Runtime, I>>()),
		(id + item, mel::<pallet_nfts::ItemConfig>()),
		(account + id + item, 0),
		(
			id + item + 1 + mel::<pallet_nfts::AttributeNamespace<Account>>() + 1 + 64,
			mel::<(
				frame_support::BoundedVec<u8, <Runtime as pallet_nfts::Config<I>>::ValueLimit>,
				u128,
			)>(),
		),
	])
}

fn price((items, bytes): (u32, u32)) -> u128 {
	deposit(items, bytes)
}

fn normal_block() -> Weight {
	RuntimeBlockWeights::get()
		.get(DispatchClass::Normal)
		.max_total
		.expect("normal class is capped; qed")
}

// The rate itself, so that a change to it shows up before the footprints do.
#[test]
fn the_deposit_rate_is_kreivos() {
	if !KUSAMA {
		return;
	}
	assert_eq!(deposit(1, 0), PER_ITEM);
	assert_eq!(deposit(0, 1), PER_BYTE);
	assert_eq!(deposit(2, 3), 2 * PER_ITEM + 3 * PER_BYTE);
}

// 0010-Q2, DEC-38: the storage part is a community's measured footprint at the deposit rate.
#[test]
fn the_storage_part_is_the_communitys_footprint_at_the_deposit_rate() {
	assert_eq!(community_footprint(), (COMMUNITY_ITEMS, COMMUNITY_BYTES));
	if KUSAMA {
		assert_eq!(CommunityStoragePartAmount::get(), price(community_footprint()));
		assert_eq!(CommunityStoragePartAmount::get(), 87_248_665_998);
	} else {
		assert_eq!(CommunityStoragePartAmount::get(), UNITS / 10);
	}
}

// 0010-Q1, 0010-Q2: two amounts whose sum is the registration payment, 0.5 KSM.
#[test]
fn the_registration_parts_sum_to_half_a_ksm() {
	assert_eq!(CommunityDepositAmount::get(), UNITS / 2);
	assert_eq!(
		CommunityStoragePartAmount::get() + CommunityConsumptionPartAmount::get(),
		CommunityDepositAmount::get()
	);
	if KUSAMA {
		assert_eq!(CommunityConsumptionPartAmount::get(), 412_751_334_002);
	}
	assert!(CommunityStoragePartAmount::get() < CommunityConsumptionPartAmount::get());
}

// 0007-A1: provisional at the existential deposit. The measured footprint is recorded, and priced
// well above it, for the economic study.
#[test]
fn the_membership_deposit_is_the_existential_deposit_whatever_the_footprint_prices_at() {
	assert_eq!(MembershipDepositAmount::get(), EXISTENTIAL_DEPOSIT);
	assert_eq!(membership_footprint(), (6, 1_286));
	if KUSAMA {
		assert_eq!(MembershipDepositAmount::get(), 33_333_333);
		assert_eq!(price(membership_footprint()), 40_428_666_198);
	}
	assert!(price(membership_footprint()) > MembershipDepositAmount::get());
}

// 0012-Q1d: fixed, at the pallet's own measured footprint of one subscription.
#[test]
fn the_subscription_deposit_is_the_subscription_footprint_at_the_deposit_rate() {
	TestExternalities::default().execute_with(|| {
		let f = crate::Listings::subscription_footprint();
		assert_eq!((f.count as u32, f.size as u32), (1, SUBSCRIPTION_BYTES));
		assert_eq!(SubscriptionDepositAmount::get(), deposit(f.count as u32, f.size as u32));
		if KUSAMA {
			assert_eq!(SubscriptionDepositAmount::get(), 6_741_666_585);
		}
	})
}

// m3-retirement-delay, REQ-MI-11: at least Kusama's unbonding (7 days) plus a margin.
#[test]
fn the_retirement_delay_is_ten_days_and_clears_kusamas_unbonding_with_margin() {
	assert_eq!(RetirementDelayInBlocks::get(), 144_000);
	assert_eq!(RetirementDelayInBlocks::get(), 10 * DAYS);
	assert!(RetirementDelayInBlocks::get() >= WEEKS + DAYS);
}

// 0009-A6, 0009-A11 and the pallet's `integrity_test`: the lead is shorter than every billing
// period (1 day, `MinBillingPeriod`) and at least a bucket.
#[test]
fn the_listings_clock_values_are_the_ruled_ones() {
	assert_eq!(ListingsDueBucketSize::get(), 10 * MINUTES);
	assert_eq!(ListingsDueBucketSize::get(), 100);
	assert_eq!(ListingsRenewalLead::get(), 12 * HOURS);
	assert_eq!(ListingsRenewalLead::get(), 7_200);
	assert!(ListingsDueBucketSize::get() <= ListingsRenewalLead::get());
	assert!(ListingsRenewalLead::get() < DAYS);
}

// Review of #515: a call that issues or retires `MaxIssuePerCall` memberships must be includable,
// with room for the block's other traffic: within half of a normal-class block's `ref_time` and
// proof size. (Weights are the pallet's estimates until `/cmd bench` replaces them.)
#[test]
fn a_full_issuance_or_retirement_call_fits_half_a_block() {
	TestExternalities::default().execute_with(|| {
		type W = crate::weights::pallet_communities_manager::WeightInfo<Runtime>;
		let (block, n) = (normal_block(), MaxIssuePerCall::get());
		assert_eq!(n, 128);
		for w in [W::issue_memberships(n), W::retire_memberships(n)] {
			assert!(w.ref_time() <= block.ref_time() / 2, "{w:?} against {block:?}");
			assert!(w.proof_size() <= block.proof_size() / 2, "{w:?} against {block:?}");
		}
		// The next power of two would not.
		let w = W::issue_memberships(2 * n);
		assert!(w.ref_time() > block.ref_time() / 2 || w.proof_size() > block.proof_size() / 2);
	})
}

// `PLAN.md` §7: a full batch of charges stays under 10 % of a block's normal `ref_time`, and a
// bucket holds at least a batch.
#[test]
fn a_full_batch_of_charges_stays_under_a_tenth_of_a_block() {
	TestExternalities::default().execute_with(|| {
		type L = crate::weights::pallet_listings::WeightInfo<Runtime>;
		let block = normal_block();
		assert!(L::process_due(ListingsMaxChargesPerBlock::get()).ref_time() <= block.ref_time() / 10);
		assert!(L::process_due(ListingsMaxDuePerBucket::get()).ref_time() <= block.ref_time() / 10);
		assert!(ListingsMaxChargesPerBlock::get() <= ListingsMaxDuePerBucket::get());
	})
}
