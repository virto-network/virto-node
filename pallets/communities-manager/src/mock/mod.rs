use super::*;

use frame_support::{
	derive_impl, parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU16, ConstU32, ConstU64, EitherOf, EqualPrivilegeOnly, Everything},
	PalletId,
};
use frame_system::{EnsureNever, EnsureRoot, EnsureRootWithSuccess, EnsureSigned};
use pallet_communities::{origin::EnsureCommunity, Tally, VoteWeight};
use parity_scale_codec::Compact;
use sp_io::TestExternalities;
use sp_runtime::{
	traits::{IdentifyAccount, IdentityLookup, Verify},
	MultiSignature,
};
pub use virto_common::{CommunityId, MembershipId};

pub use crate as pallet_communities_manager;

mod collective;
mod weights;
pub use weights::*;

#[cfg(feature = "runtime-benchmarks")]
mod runtime_benchmarks;
#[cfg(feature = "runtime-benchmarks")]
use runtime_benchmarks::*;

type Block = frame_system::mocking::MockBlock<Test>;
type WeightInfo = ();

pub type AccountPublic = <MultiSignature as Verify>::Signer;
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
pub type Balance = u64;
pub type AssetId = u32;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Assets: pallet_assets,
		Balances: pallet_balances,
		CollectiveReferenda: pallet_referenda::<Instance1>,
		Collective: pallet_ranked_collective::<Instance1>,
		Scheduler: pallet_scheduler,
		Referenda: pallet_referenda,
		Tracks: pallet_referenda_tracks,
		Memberships: pallet_nfts,
		Communities: pallet_communities,
		CommunitiesManager: pallet_communities_manager,
	}
);

parameter_types! {
	pub const RootAccount: AccountId = AccountId::new([0xff; 32]);
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type Block = Block;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type AccountData = pallet_balances::AccountData<Balance>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig as pallet_balances::DefaultConfig)]
impl pallet_balances::Config for Test {
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type FreezeIdentifier = RuntimeFreezeReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
}

#[derive_impl(pallet_assets::config_preludes::TestDefaultConfig as pallet_assets::DefaultConfig)]
impl pallet_assets::Config for Test {
	type Balance = Balance;
	type AssetIdParameter = Compact<AssetId>;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<Self::AccountId>>;
	type ForceOrigin = EnsureRoot<Self::AccountId>;
	type Freezer = ();
	type RemoveItemsLimit = ConstU32<1000>;
	type RuntimeHoldReason = RuntimeHoldReason;
}

impl pallet_scheduler::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Self>;
	type Preimages = ();
}

parameter_types! {
	pub static AlarmInterval: u64 = 1;
	pub const MaxTracks: u32 = u32::MAX;
}

impl pallet_referenda::Config for Test {
	type WeightInfo = ();
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type Scheduler = Scheduler;
	type Currency = Balances;
	type SubmitOrigin = EnsureSigned<AccountId>;
	type CancelOrigin = EnsureRoot<AccountId>;
	type KillOrigin = EnsureRoot<AccountId>;
	type Slash = ();
	type Votes = VoteWeight;
	type Tally = Tally<Test>;
	type SubmissionDeposit = ConstU64<2>;
	type MaxQueued = ConstU32<3>;
	type UndecidingTimeout = ConstU64<20>;
	type AlarmInterval = AlarmInterval;
	type Tracks = Tracks;
	type Preimages = ();
}

impl pallet_referenda_tracks::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type TrackId = CommunityId;
	type MaxTracks = MaxTracks;
	type AdminOrigin = EnsureRoot<AccountId>;
	type UpdateOrigin = EnsureRoot<AccountId>;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = TracksBenchmarkHelper;
}

type Deposit = Option<(Balance, AccountId, AccountId)>;
parameter_types! {
	pub const CommunitiesPalletId: PalletId = PalletId(*b"kv/comms");
	pub const MembershipsManagerCollectionId: CommunityId = 0;
	pub const MembershipNftAttr: &'static [u8; 10] = b"membership";
	pub const TestCommunity: CommunityId = 1;
	pub const NoDepositOnRootRegistration: Deposit = None;
}

impl pallet_nfts::Config for Test {
	type ApprovalsLimit = ();
	type AttributeDepositBase = ();
	type CollectionDeposit = ();
	type CollectionId = CommunityId;
	type CreateOrigin =
		AsEnsureOriginWithArg<EitherOf<EnsureRootWithSuccess<AccountId, RootAccount>, EnsureSigned<AccountId>>>;
	type Currency = Balances;
	type DepositPerByte = ();
	type Features = ();
	type ForceOrigin = EnsureRoot<AccountId>;
	type ItemAttributesApprovalsLimit = ();
	type ItemDeposit = ();
	type ItemId = MembershipId;
	type KeyLimit = ConstU32<64>;
	type Locker = ();
	type MaxAttributesPerCall = ();
	type MaxDeadlineDuration = ();
	type MaxTips = ();
	type MetadataDepositBase = ();
	type OffchainPublic = AccountPublic;
	type OffchainSignature = MultiSignature;
	type RuntimeEvent = RuntimeEvent;
	type StringLimit = ();
	type ValueLimit = ConstU32<10>;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
}

impl pallet_communities::Config for Test {
	type PalletId = CommunitiesPalletId;
	type CommunityId = CommunityId;
	type MembershipId = MembershipId;
	type Assets = Assets;
	type Balances = Balances;
	type ItemConfig = pallet_nfts::ItemConfig;
	type MemberMgmt = Memberships;
	type Polls = Referenda;
	type CreateOrigin = EnsureNever<Deposit>;
	type AdminOrigin = EnsureCommunity<Self>;
	type MemberMgmtOrigin = EnsureCommunity<Self>;
	type RuntimeCall = RuntimeCall;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type WeightInfo = WeightInfo;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = CommunityBenchmarkHelper;
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	// Types to support community creation
	type CreateCollection = Memberships;
	type Tracks = Tracks;
	type RankedCollective = Collective;
	type RegisterOrigin = EnsureRootWithSuccess<AccountId, NoDepositOnRootRegistration>;
	// Types to support memberships creation
	type CreateMembershipsOrigin = EnsureRoot<AccountId>;
	type MembershipId = MembershipId;

	type MembershipsManagerCollectionId = MembershipsManagerCollectionId;
	type MembershipsManagerOwner = RootAccount;
	type CreateMemberships = Memberships;

	type WeightInfo = WeightInfo;
}

#[allow(dead_code)]
fn new_test_ext() -> TestExternalities {
	TestExternalities::new(Default::default())
}
