use frame_support::{
	parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU128, ConstU16, ConstU32, ConstU64, EqualPrivilegeOnly},
	weights::Weight,
	PalletId,
};
use frame_system::{EnsureRoot, EnsureSigned};
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

use crate as pallet_communities;

type Block = frame_system::mocking::MockBlock<Test>;
type WeightInfo = ();

pub type AccountId = u64;
pub type Balance = u128;
pub type AssetId = u32;
pub type CommunityId = u128;

pub type MembershipRank = u32;
pub type MembershipPassport = ();
pub type VoteWeight = u128;

impl pallet_communities::traits::rank::MemberRank<MembershipRank> for MembershipPassport {
	fn rank(&self) -> u32 {
		0
	}
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		Assets: pallet_assets,
		Balances: pallet_balances,
		Communities: pallet_communities,
		Preimage: pallet_preimage,
		Scheduler: pallet_scheduler,
		System: frame_system,
	}
);

impl<T: pallet_communities::Config>
	Into<Result<frame_system::RawOrigin<T::AccountId>, pallet_communities::RawOrigin<T>>>
	for pallet_communities::RawOrigin<T>
{
	fn into(self) -> Result<frame_system::RawOrigin<T::AccountId>, pallet_communities::RawOrigin<T>> {
		Ok(frame_system::RawOrigin::Signed(
			pallet_communities::Pallet::<T>::get_community_account_id(&self.community_id),
		))
	}
}

impl<T: pallet_communities::Config> From<pallet_communities::RawOrigin<T>> for frame_system::RawOrigin<T::AccountId> {
	fn from(o: pallet_communities::RawOrigin<T>) -> frame_system::RawOrigin<T::AccountId> {
		frame_system::RawOrigin::Signed(pallet_communities::Pallet::<T>::get_community_account_id(
			&o.community_id,
		))
	}
}

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
	type AssetIdParameter = codec::Compact<u32>;
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
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Weight::from_parts(1_000_000_000, 1_048_576);
	pub const MaxScheduledPerBlock: u32 = 512;
	pub const PreimageBaseDeposit: u64 = 2;
	pub const PreimageByteDeposit: u64 = 1;
}

impl pallet_preimage::Config for Test {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ManagerOrigin = EnsureSigned<AccountId>;
	type BaseDeposit = PreimageBaseDeposit;
	type ByteDeposit = PreimageByteDeposit;
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
	pub const CommunitiesMetadataUrlSize: u32 = 32;
	#[derive(Debug, Clone, PartialEq)]
	pub const CommunitiesMaxUrls: u32 = 5;
	#[derive(Debug, Clone, PartialEq)]
	pub const CommunitiesMaxLocations: u32 = 2;
	#[derive(Debug, Clone, PartialEq)]
	pub const CommunitiesMaxProposals: u32 = 2;
}

impl pallet_communities::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = WeightInfo;
	type Assets = Assets;
	type Balances = Balances;
	type CommunityId = CommunityId;

	type MembershipRank = MembershipRank;
	type MembershipPassport = MembershipPassport;
	type VoteWeight = VoteWeight;

	type PalletId = CommunitiesPalletId;
	type FreezeIdentifier = <Test as pallet_balances::Config>::FreezeIdentifier;
	type MetadataUrlSize = CommunitiesMetadataUrlSize;
	type MaxUrls = CommunitiesMaxUrls;
	type MaxLocations = CommunitiesMaxLocations;
	type Preimage = Preimage;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxProposals = CommunitiesMaxProposals;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap()
		.into()
}
