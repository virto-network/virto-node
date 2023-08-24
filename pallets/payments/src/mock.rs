pub use crate as pallet_payments;
pub use crate::types::*;
use frame_support::{
	parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU32, ConstU64},
	PalletId,
};

use sp_core::H256;
use sp_keystore::{testing::MemoryKeystore, KeystoreExt};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BoundedVec, BuildStorage, Percent,
};

type Block = frame_system::mocking::MockBlock<Test>;
type AccountId = u64;

pub const SENDER_ACCOUNT: AccountId = 10;
pub const PAYMENT_BENEFICIARY: AccountId = 11;
pub const ASSET_ADMIN_ACCOUNT: AccountId = 1;
pub const ASSET_ID: u32 = 1;
pub const RESOLVER_ACCOUNT: AccountId = 12;
pub const INCENTIVE_PERCENTAGE: u8 = 10;
pub const MARKETPLACE_FEE_PERCENTAGE: u8 = 10;
pub const INITIAL_BALANCE: u64 = 100;
/* for future uses
pub const PAYMENT_RECIPENT_FEE_CHARGED: AccountId = 21;
pub const CANCEL_BLOCK_BUFFER: u64 = 600;
*/
/// Destination account for the fee payment
pub const FEE_SENDER_ACCOUNT: AccountId = 30;
pub const FEE_BENEFICIARY_ACCOUNT: AccountId = 31;
pub const FEE_SYSTEM_ACCOUNT: AccountId = 32;

pub const SYSTEM_FEE: u64 = 2;
pub const EXPECTED_SYSTEM_TOTAL_FEE: u64 = 4;
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
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
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

pub struct MockFeeHandler;

impl crate::types::FeeHandler<Test> for MockFeeHandler {
	fn apply_fees(
		_sender: &AccountId,
		_beneficiary: &AccountId,
		amount: &Balance,
		_remark: Option<&[u8]>,
	) -> Fees<Test> {
		let sender_fees = vec![
			SubTypes::Fixed(FEE_SENDER_ACCOUNT, FEE_SENDER_AMOUNT),
			SubTypes::Percentage(FEE_SYSTEM_ACCOUNT, Percent::from_percent(MARKETPLACE_FEE_PERCENTAGE)),
		];

		let beneficiary_fees = vec![
			SubTypes::Fixed(FEE_BENEFICIARY_ACCOUNT, FEE_BENEFICIARY_AMOUNT),
			SubTypes::Percentage(FEE_SYSTEM_ACCOUNT, Percent::from_percent(MARKETPLACE_FEE_PERCENTAGE)),
		];

		let compute_fee = |fees: &Vec<SubTypes<Test>>| -> FeeDetails<Test> {
			let details = fees
				.iter()
				.map(|fee| match fee {
					SubTypes::Fixed(account, amount_fixed) => (account.clone(), *amount_fixed),
					SubTypes::Percentage(account, percent) => (account.clone(), percent.mul_floor(*amount)),
				})
				.collect::<Vec<(AccountId, Balance)>>();
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

pub struct MockDisputeResolver;
impl crate::types::DisputeResolver<AccountId> for MockDisputeResolver {
	fn get_resolver_account() -> AccountId {
		RESOLVER_ACCOUNT
	}
}

parameter_types! {
	pub const MaxRemarkLength: u32 = 50;
	pub const IncentivePercentage: Percent = Percent::from_percent(INCENTIVE_PERCENTAGE);
	pub const PaymentPalletId: PalletId = PalletId(*b"payments");
}

impl pallet_payments::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Assets = Assets;
	type AssetsBalance = u64;
	type FeeHandler = MockFeeHandler;
	type IncentivePercentage = IncentivePercentage;
	type MaxRemarkLength = MaxRemarkLength;
	type DisputeResolver = MockDisputeResolver;
	type PalletId = PaymentPalletId;
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxDiscounts = ConstU32<50>;
	type MaxFees = ConstU32<50>;
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

	let mut ext = sp_io::TestExternalities::new(t);
	ext.register_extension(KeystoreExt::new(MemoryKeystore::new()));
	ext.execute_with(|| System::set_block_number(1));
	ext
}
