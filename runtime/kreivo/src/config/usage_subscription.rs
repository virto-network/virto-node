//! Usage subscriptions: a group buys a recurring, shared allowance of weight for its members
//! (`DEC-24`, `DEC-25`). Values are those of `deployment/kreivo-0.18.md`.

use super::*;

use frame_support::traits::{Contains, EitherOf};
use pallet_communities::{origin::EnsureCommunity, types::CommunityState};
use runtime_constants::time::WEEKS;
use sp_runtime::Perbill;

use communities::{memberships::CommunityAccount, EnsureCommunityAccount};
use governance::pallet_custom_origins::{UsageSubscriptionAdmin, UsageSubscriptionAmender};

parameter_types! {
	/// The collective's inventory: merchant 0, the collective's own community, inventory 0.
	pub const UsageOfferInventory: (CommunityId, u32) = (0, 0);
	/// The shortest usage period: 1 hour.
	pub const MinUsagePeriod: BlockNumber = HOURS;
	/// The shortest billing period: 1 day. Longer than listings' renewal lead (12 hours).
	pub const MinBillingPeriod: BlockNumber = DAYS;
	pub const MaxTrialPeriods: u32 = 3;
	pub const MaxMembershipScan: u32 = 4;
	pub const MaxPayingGroupChanges: u32 = 3;
	pub const MaxOffersPerPage: u32 = 50;
	/// Allowances are normalized to a week of relay chain blocks (`DEC-39`).
	pub const CapacityPeriod: BlockNumber = WEEKS;
	/// The load share that raises the capacity alert (m3-alert-threshold).
	pub const CapacityAlertThreshold: Perbill = Perbill::from_percent(80);
}

/// A community that is `Active`, and not the collective (community 0): `REQ-GR-4`.
pub struct ActiveCommunity;

impl Contains<CommunityId> for ActiveCommunity {
	fn contains(id: &CommunityId) -> bool {
		*id != 0 && Communities::community_state(id) == Some(CommunityState::Active)
	}
}

/// Offers, terminations and capacity: Root or track 3 (`DEC-10`, m3-capacity-origin).
pub type UsageSubscriptionAdminOrigin = EitherOf<EnsureRoot<AccountId>, UsageSubscriptionAdmin>;
/// Amendments: Root or track 5 (`DEC-28`).
pub type UsageSubscriptionAmendOrigin = EitherOf<EnsureRoot<AccountId>, UsageSubscriptionAmender>;

impl pallet_usage_subscription::Config for Runtime {
	type WeightInfo = pallet_usage_subscription::SubstrateWeight<Runtime>;
	type Memberships = Memberships;
	type GroupAccount = CommunityAccount;
	type UsableGroup = ActiveCommunity;
	type Subscriptions = Listings;
	type OfferInventory = UsageOfferInventory;
	type Payee = TreasuryAccount;
	type StandardOfferOrigin = UsageSubscriptionAdminOrigin;
	type CustomOfferOrigin = UsageSubscriptionAdminOrigin;
	type TerminateOrigin = UsageSubscriptionAdminOrigin;
	type AmendOrigin = UsageSubscriptionAmendOrigin;
	type GroupOrigin = EitherOf<EnsureCommunity<Runtime>, EnsureCommunityAccount>;
	type CapacityOrigin = UsageSubscriptionAdminOrigin;
	type BlockNumberProvider = RelaychainData;
	type MinUsagePeriod = MinUsagePeriod;
	type MinBillingPeriod = MinBillingPeriod;
	type MaxTrialPeriods = MaxTrialPeriods;
	type MaxMembershipScan = MaxMembershipScan;
	type MaxPayingGroupChanges = MaxPayingGroupChanges;
	type MaxOffersPerPage = MaxOffersPerPage;
	type CapacityPeriod = CapacityPeriod;
	type CapacityAlertThreshold = CapacityAlertThreshold;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = benchmarks::UsageSubscriptionBenchmarkHelper;
}

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks {
	use super::*;
	use crate::config::listings_orders::{ListingsInstance, ListingsMaxDuePerBucket};
	use frame_contrib_traits::memberships::Manager;
	use frame_support::traits::Footprint;
	use frame_support::traits::{
		fungible::Mutate as _,
		fungibles::{Create, Inspect, Mutate},
		Consideration,
	};
	use pallet_usage_subscription::{GroupOf, MembershipOf};
	use sp_runtime::traits::StaticLookup;

	const ASSET: FungibleAssetLocation = FungibleAssetLocation::Here(1);

	/// Registers `id` as a community with funds, if it is not one yet.
	fn community(id: CommunityId) -> AccountId {
		if Communities::community_state(&id).is_none() {
			frame_support::assert_ok!(CommunitiesManager::register(
				RuntimeOrigin::root(),
				id,
				BoundedVec::try_from(b"Benchmark".to_vec()).expect("meets max length; qed"),
				<system::CommunityLookup as StaticLookup>::unlookup(AccountId::new([id as u8; 32])),
				None,
				None,
			));
		}
		let account = Communities::community_account(&id);
		Balances::mint_into(&account, 1_000_000 * UNITS).expect("minting is expected to succeed; qed");
		account
	}

	fn asset() {
		if !Assets::asset_exists(ASSET) {
			<Assets as Create<_>>::create(ASSET, TreasuryAccount::get(), true, EXISTENTIAL_DEPOSIT)
				.expect("creating an asset class is expected to succeed; qed");
		}
	}

	pub struct UsageSubscriptionBenchmarkHelper;

	impl pallet_usage_subscription::BenchmarkHelper<Runtime> for UsageSubscriptionBenchmarkHelper {
		fn group() -> GroupOf<Runtime> {
			let account = community(1);
			asset();
			Assets::mint_into(ASSET, &account, 1_000_000 * UNITS).expect("minting is expected to succeed; qed");
			1
		}

		fn group_origin(group: &GroupOf<Runtime>) -> RuntimeOrigin {
			RuntimeOrigin::signed(Communities::community_account(group))
		}

		fn add_member(group: &GroupOf<Runtime>, who: &AccountId) -> MembershipOf<Runtime> {
			let account = Communities::community_account(group);
			frame_support::assert_ok!(CommunitiesManager::issue_memberships(RuntimeOrigin::signed(account), 1));
			let membership = pallet_communities_manager::NextMembershipId::<Runtime>::get() - 1;
			<Memberships as Manager<AccountId>>::assign(group, &membership, who)
				.expect("assigning an issued membership is expected to succeed; qed");
			membership
		}

		fn price() -> pallet_listings::ItemPrice<FungibleAssetLocation, Balance> {
			asset();
			pallet_listings::ItemPrice {
				asset: ASSET,
				amount: 100 * EXISTENTIAL_DEPOSIT,
			}
		}

		fn setup_payee() {
			Balances::mint_into(&TreasuryAccount::get(), 1_000_000 * UNITS)
				.expect("minting is expected to succeed; qed");
		}

		fn fund_subscriber(who: &AccountId) {
			<<Runtime as pallet_listings::Config<ListingsInstance>>::SubscriptionDeposit as Consideration<
				AccountId,
				Footprint,
			>>::ensure_successful(who, Listings::subscription_footprint());
		}

		fn stale_group() -> GroupOf<Runtime> {
			community(2);
			2
		}

		fn due_queue_capacity() -> u32 {
			pallet_listings::MAX_QUEUE_PROBES * ListingsMaxDuePerBucket::get()
		}

		fn due_overflow_is_empty() -> bool {
			pallet_listings::DueOverflow::<Runtime, ListingsInstance>::iter()
				.next()
				.is_none()
		}
	}
}
