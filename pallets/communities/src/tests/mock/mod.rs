mod poll;

use frame_support::{
	ord_parameter_types, parameter_types,
	traits::{
		fungible::HoldConsideration, AsEnsureOriginWithArg, ConstU128, ConstU16, ConstU32, ConstU64,
		EqualPrivilegeOnly, Footprint,
	},
	weights::Weight,
	PalletId,
};
use frame_system::{EnsureRoot, EnsureSigned, EnsureSignedBy};
use parity_scale_codec::Compact;
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, Convert, IdentityLookup},
	BuildStorage,
};

use crate::{
	self as pallet_communities,
	types::{Tally, VoteWeight},
};

use poll::TestPolls;

type Block = frame_system::mocking::MockBlock<Test>;
type WeightInfo = ();

pub type AccountId = u64;
pub type Balance = u128;
pub type AssetId = u32;
pub type CommunityId = u128;
pub type MembershipPassport = ();

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		Assets: pallet_assets,
		Balances: pallet_balances,
		Communities: pallet_communities,
		Preimage: pallet_preimage,
		Referenda: pallet_referenda,
		Tracks: pallet_referenda_tracks,
		Scheduler: pallet_scheduler,
		System: frame_system,
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_assets::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type AssetIdParameter = Compact<u32>;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<Self::AccountId>>;
	type ForceOrigin = EnsureRoot<Self::AccountId>;
	type AssetDeposit = ConstU128<100>;
	type AssetAccountDeposit = ConstU128<1>;
	type MetadataDepositBase = ConstU128<10>;
	type MetadataDepositPerByte = ConstU128<1>;
	type ApprovalDeposit = ConstU128<1>;
	type StringLimit = ConstU32<50>;
	type Freezer = ();
	type Extra = ();
	type CallbackHandle = ();
	type WeightInfo = WeightInfo;
	type RemoveItemsLimit = ConstU32<1000>;
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxHolds = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type WeightInfo = WeightInfo;
	type MaxLocks = ConstU32<10>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = ();
	type FreezeIdentifier = ();
	type MaxHolds = ConstU32<10>;
	type MaxFreezes = ConstU32<10>;
	type RuntimeFreezeReason = ();
}

parameter_types! {
		pub static AlarmInterval: u64 = 1;
}
ord_parameter_types! {
	   pub const One: u64 = 1;
}
impl pallet_referenda::Config for Test {
	type WeightInfo = ();
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type Scheduler = Scheduler;
	type Currency = pallet_balances::Pallet<Self>;
	type SubmitOrigin = frame_system::EnsureSigned<u64>;
	type CancelOrigin = EnsureSignedBy<One, u64>;
	type KillOrigin = EnsureRoot<u64>;
	type Slash = ();
	type Votes = VoteWeight;
	type Tally = Tally<Test>;
	type SubmissionDeposit = ConstU128<2>;
	type MaxQueued = ConstU32<3>;
	type UndecidingTimeout = ConstU64<20>;
	type AlarmInterval = AlarmInterval;
	type Tracks = Tracks;
	type Preimages = Preimage;
}
impl pallet_referenda_tracks::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type TrackId = CommunityId;
	type MaxTracks = ConstU32<2>;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Weight::from_parts(1_000_000_000, 1_048_576);
	pub const MaxScheduledPerBlock: u32 = 512;
}
pub struct ConvertDeposit;
impl Convert<Footprint, u128> for ConvertDeposit {
	fn convert(a: Footprint) -> u128 {
		(a.count * 2 + a.size).into()
	}
}
impl pallet_preimage::Config for Test {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ManagerOrigin = EnsureSigned<AccountId>;
	type Consideration = HoldConsideration<u64, Balances, (), ConvertDeposit>;
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
	type Preimages = Preimage;
}

parameter_types! {
	pub const CommunitiesPalletId: PalletId = PalletId(*b"kv/comms");
	#[derive(Debug, Clone, PartialEq)]
	pub const CommunitiesMaxProposals: u32 = 2;
}

impl pallet_communities::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = WeightInfo;
	type Assets = Assets;
	type Balances = Balances;
	type CommunityId = CommunityId;
	type Membership = MembershipPassport;
	type PalletId = CommunitiesPalletId;
	type Polls = Referenda;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap()
		.into()
}
