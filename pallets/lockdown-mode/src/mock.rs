use crate as pallet_lockdown_mode;
use crate::impls::PauseXcmExecution;
use cumulus_primitives_core::{relay_chain::BlockNumber as RelayBlockNumber, DmpMessageHandler};
use frame_support::{
	traits::{ConstU64, Contains},
	weights::Weight,
};

use sp_core::H256;
use sp_keystore::{testing::MemoryKeystore, KeystoreExt};
use sp_runtime::{
	traits::{BlakeTwo256, ConstU32, IdentityLookup},
	BuildStorage, DispatchResult,
};

type Block = frame_system::mocking::MockBlock<Test>;

pub type AccountId = u64;

frame_support::parameter_types! {
	pub const StatemineParaIdInfo: u32 = 1000u32;
	pub const StatemineAssetsInstanceInfo: u8 = 50u8;
	pub const StatemineAssetIdInfo: u128 = 1u128;
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		LockdownMode: pallet_lockdown_mode::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Remark: pallet_remark::{Pallet, Call, Storage, Event<T>},
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
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
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type Balance = u64;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type MaxHolds = ();
}
pub struct RuntimeBlackListedCalls;
impl Contains<RuntimeCall> for RuntimeBlackListedCalls {
	fn contains(call: &RuntimeCall) -> bool {
		matches!(call, RuntimeCall::Balances(_))
	}
}

pub struct LockdownDmpHandler;
impl DmpMessageHandler for LockdownDmpHandler {
	fn handle_dmp_messages(_iter: impl Iterator<Item = (RelayBlockNumber, Vec<u8>)>, limit: Weight) -> Weight {
		limit
	}
}

pub struct XcmExecutionManager {}

impl PauseXcmExecution for XcmExecutionManager {
	fn suspend_xcm_execution() -> DispatchResult {
		Ok(())
	}
	fn resume_xcm_execution() -> DispatchResult {
		Ok(())
	}
}

impl pallet_remark::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

impl pallet_lockdown_mode::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type LockdownModeOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type BlackListedCalls = RuntimeBlackListedCalls;
	type LockdownDmpHandler = LockdownDmpHandler;
	type XcmExecutorManager = XcmExecutionManager;
	type WeightInfo = pallet_lockdown_mode::weights::SubstrateWeight<Test>;
}

pub fn new_test_ext(initial_status: bool) -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	pallet_lockdown_mode::GenesisConfig::<Test> {
		initial_status,
		_config: Default::default(),
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.register_extension(KeystoreExt::new(MemoryKeystore::new()));
	ext.execute_with(|| System::set_block_number(1));
	ext
}
