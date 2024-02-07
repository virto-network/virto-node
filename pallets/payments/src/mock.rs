pub use crate as pallet_payments;
pub use crate::types::*;
use frame_support::{
	parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU32, ConstU64, EqualPrivilegeOnly, OnFinalize, OnInitialize},
	weights::Weight,
	PalletId,
};

use frame_system::EnsureRoot;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_keystore::{testing::MemoryKeystore, KeystoreExt};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BoundedVec, BuildStorage, Percent,
};

type Block = frame_system::mocking::MockBlock<Test>;
type AccountId = u64;
#[allow(unused)]
type AssetId = u32;

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct PaymentId(pub u32);

pub const SENDER_ACCOUNT: AccountId = 10;
pub const PAYMENT_BENEFICIARY: AccountId = 11;
pub const ASSET_ADMIN_ACCOUNT: AccountId = 3;
pub const ROOT_ACCOUNT: AccountId = 1;

pub const ASSET_ID: u32 = 1;
pub const INCENTIVE_PERCENTAGE: u8 = 10;
pub const MARKETPLACE_FEE_PERCENTAGE: u8 = 15;
pub const INITIAL_BALANCE: u64 = 100;
pub const PAYMENT_ID: PaymentId = PaymentId(1);

pub const FEE_SENDER_ACCOUNT: AccountId = 30;
pub const FEE_BENEFICIARY_ACCOUNT: AccountId = 31;
pub const FEE_SYSTEM_ACCOUNT: AccountId = 32;

pub const SYSTEM_FEE: u64 = 3;
pub const EXPECTED_SYSTEM_TOTAL_FEE: u64 = 6;
pub const EXPECTED_SYSTEM_SENDER_FEE: u64 = 3; // 15% of 20

pub const FEE_SENDER_AMOUNT: Balance = 2;
pub const FEE_BENEFICIARY_AMOUNT: Balance = 3;
pub const PAYMENT_AMOUNT: u64 = 20;
pub const INCENTIVE_AMOUNT: u64 = PAYMENT_AMOUNT / INCENTIVE_PERCENTAGE as u64;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		Assets: pallet_assets,
		Sudo: pallet_sudo,
		Payments: pallet_payments,
		Scheduler: pallet_scheduler,
		Preimage: pallet_preimage,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub MaxWeight: Weight = Weight::from_parts(2_000_000_000_000, u64::MAX);

}

pub type Balance = u64;

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
	type Balance = Balance;
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
	type RuntimeFreezeReason = RuntimeFreezeReason;
}

impl pallet_assets::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = u32;
	type AssetIdParameter = u32;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<u64>>;
	type ForceOrigin = frame_system::EnsureRoot<u64>;
	type AssetDeposit = ConstU64<1>;
	type AssetAccountDeposit = ConstU64<10>;
	type MetadataDepositBase = ConstU64<1>;
	type MetadataDepositPerByte = ConstU64<1>;
	type ApprovalDeposit = ConstU64<1>;
	type StringLimit = ConstU32<50>;
	type Freezer = ();
	type WeightInfo = ();
	type Extra = ();
	type RemoveItemsLimit = ConstU32<5>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
	type CallbackHandle = ();
	type MaxHolds = ConstU32<50>;
	type RuntimeHoldReason = RuntimeHoldReason;
}

impl pallet_sudo::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = ();
}

impl pallet_preimage::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<u64>;
	type Consideration = ();
}

impl pallet_scheduler::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaxWeight;
	type ScheduleOrigin = EnsureRoot<u64>;
	type MaxScheduledPerBlock = ConstU32<100>;
	type WeightInfo = ();
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type Preimages = Preimage;
}

pub struct MockFeeHandler;

const MANDATORY_FEE: bool = true;

impl crate::types::FeeHandler<Test> for MockFeeHandler {
	fn apply_fees(
		_asset: &AssetIdOf<Test>,
		_sender: &AccountId,
		_beneficiary: &AccountId,
		amount: &Balance,
		_remark: Option<&[u8]>,
	) -> Fees<Test> {
		let sender_fees = vec![
			SubTypes::Fixed(FEE_SENDER_ACCOUNT, FEE_SENDER_AMOUNT, !MANDATORY_FEE),
			SubTypes::Percentage(
				FEE_SYSTEM_ACCOUNT,
				Percent::from_percent(MARKETPLACE_FEE_PERCENTAGE),
				MANDATORY_FEE,
			),
		];

		let beneficiary_fees = vec![
			SubTypes::Fixed(FEE_BENEFICIARY_ACCOUNT, FEE_BENEFICIARY_AMOUNT, !MANDATORY_FEE),
			SubTypes::Percentage(
				FEE_SYSTEM_ACCOUNT,
				Percent::from_percent(MARKETPLACE_FEE_PERCENTAGE),
				MANDATORY_FEE,
			),
		];

		let compute_fee = |fees: &Vec<SubTypes<Test>>| -> FeeDetails<Test> {
			let details = fees
				.iter()
				.map(|fee| match fee {
					SubTypes::Fixed(account, amount_fixed, charged_disputes) => {
						(*account, *amount_fixed, *charged_disputes)
					}
					SubTypes::Percentage(account, percent, charged_disputes) => {
						(*account, percent.mul_floor(*amount), *charged_disputes)
					}
				})
				.collect::<Vec<(AccountId, Balance, bool)>>();
			// This is a test, so i'm just unwrapping
			let bounded_details: FeeDetails<Test> = BoundedVec::try_from(details).unwrap();
			bounded_details
		};

		Fees {
			sender_pays: compute_fee(&sender_fees),
			beneficiary_pays: compute_fee(&beneficiary_fees),
		}
	}
}

#[cfg(feature = "runtime-benchmarks")]
pub struct BenchmarkHelper;
#[cfg(feature = "runtime-benchmarks")]
impl super::BenchmarkHelper<AccountId, AssetId, Balance> for BenchmarkHelper {
	fn create_asset(id: AssetId, admin: AccountId, is_sufficient: bool, min_balance: Balance) {
		<Assets as frame_support::traits::tokens::fungibles::Create<AccountId>>::create(
			id,
			admin,
			is_sufficient,
			min_balance,
		)
		.unwrap();
	}
}

parameter_types! {
	pub const MaxRemarkLength: u8 = 50;
	pub const IncentivePercentage: Percent = Percent::from_percent(INCENTIVE_PERCENTAGE);
	pub const PaymentPalletId: PalletId = PalletId(*b"payments");
}

impl pallet_payments::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Assets = Assets;
	type AssetsBalance = u64;
	type PaymentId = PaymentId;
	type FeeHandler = MockFeeHandler;
	type IncentivePercentage = IncentivePercentage;
	type MaxRemarkLength = MaxRemarkLength;
	type DisputeResolver = frame_system::EnsureRootWithSuccess<u64, ConstU64<ROOT_ACCOUNT>>;
	type PalletId = PaymentPalletId;
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxDiscounts = ConstU32<50>;
	type MaxFees = ConstU32<50>;
	type RuntimeCall = RuntimeCall;
	type Scheduler = Scheduler;
	type Preimages = ();
	type CancelBufferBlockLength = ConstU64<10>;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = BenchmarkHelper;
}

// Build genesis storage according to the mock runtime.
pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			// id, owner, is_sufficient, min_balance
			(FEE_SENDER_ACCOUNT, INITIAL_BALANCE),
			(FEE_BENEFICIARY_ACCOUNT, INITIAL_BALANCE),
			(FEE_SYSTEM_ACCOUNT, INITIAL_BALANCE),
			(PAYMENT_BENEFICIARY, INITIAL_BALANCE),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	pallet_assets::GenesisConfig::<Test> {
		assets: vec![
			// id, owner, is_sufficient, min_balance
			(ASSET_ID, ASSET_ADMIN_ACCOUNT, true, 1),
		],
		metadata: vec![
			// id, name, symbol, decimals
			(ASSET_ID, "Token Name".into(), "TOKEN".into(), 10),
		],
		accounts: vec![
			// id, account_id, balance
			(ASSET_ID, SENDER_ACCOUNT, 100),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	pallet_sudo::GenesisConfig::<Test> {
		key: Some(ROOT_ACCOUNT),
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.register_extension(KeystoreExt::new(MemoryKeystore::new()));
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		Scheduler::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		Scheduler::on_initialize(System::block_number());
	}
}

use core::cell::Cell;
thread_local! {
	pub static LAST_ID: Cell<u32>  = const { Cell::new(0) };
}
impl pallet_payments::PaymentId<Test> for PaymentId {
	fn next(_sender: &AccountId, _beneficiary: &AccountId) -> Option<Self> {
		LAST_ID.with(|id| {
			let new_id = id.get() + 1;
			id.set(new_id);
			Some(PaymentId(new_id))
		})
	}
}
