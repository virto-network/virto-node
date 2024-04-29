use crate as pallet_fee_handler;
use frame_support::{
	pallet_prelude::Weight,
	traits::{ConstU128, ConstU16, ConstU32, ConstU64},
};

// This is horrible, I need to modify it eventually, could be handled with a call to te pallet
pub const CONST_MAX_CREDIT_USER_CAN_USE_PER_ERA: u64 = 10_000_000;
pub const CONST_MAX_GLOBAL_TOTAL_CREDIT_TO_USE_PER_ERA: u64 = 1_000_000_000;
pub const CONST_CONVERTION_RATE: u32 = 1_000;
pub const CONST_BLOCKS_OF_ERA: u64 = 100;

use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, Convert, IdentityLookup},
	BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		FeeHandler: pallet_fee_handler
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
	type AccountId = u64;
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

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ConstU32<10>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = ();
	type FreezeIdentifier = ();
	type MaxHolds = ConstU32<10>;
	type MaxFreezes = ConstU32<10>;
	type RuntimeFreezeReason = RuntimeFreezeReason;
}

impl pallet_fee_handler::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type NativeBalance = Balances;
	type RuntimeCall = RuntimeCall;
	type MaxCreditUserCanUsePerRound = ConstU64<CONST_MAX_CREDIT_USER_CAN_USE_PER_ERA>;
	type MaxGlobalTotalCreditToUsePerRound = ConstU64<CONST_MAX_GLOBAL_TOTAL_CREDIT_TO_USE_PER_ERA>;
	type ConvertionRateLockedToWeight = ConstU32<CONST_CONVERTION_RATE>;
	type BlocksOfEra = ConstU64<CONST_BLOCKS_OF_ERA>;
	type ConvertBalanceToWeight = BalanceToWeight;
}

pub struct BalanceToWeight;
impl Convert<Balance, Weight> for BalanceToWeight {
	fn convert(a: Balance) -> Weight {
		Weight::from_all(a.try_into().unwrap_or(u64::MAX))
	}
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}
