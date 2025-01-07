use super::*;

use fc_traits_gas_tank::{NonFungibleGasTank, SelectNonFungibleItem};

use pallet_asset_tx_payment::FungiblesAdapter;
use pallet_assets::BalanceToAssetBalance;
use pallet_transaction_payment::FungibleAdapter;
use runtime_common::impls::AssetsToBlockAuthor;
use virto_common::MembershipId;

// #[runtime::pallet_index(10)]
// pub type Balances
parameter_types! {
	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = ConstU32<50>;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type FreezeIdentifier = RuntimeFreezeReason;
	type MaxFreezes = ConstU32<256>;
	type RuntimeFreezeReason = RuntimeFreezeReason;
}

// #[runtime::pallet_index(11)]
// pub type TransactionPayment
parameter_types! {
	/// Relay Chain `TransactionByteFee` / 10
	pub const TransactionByteFee: Balance = 10 * MILLICENTS;
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = FungibleAdapter<Balances, ResolveTo<TreasuryAccount, Balances>>;
	type WeightToFee = WeightToFee;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type OperationalFeeMultiplier = ConstU8<5>;
}

// #[runtime::pallet_index(12)]
// pub type AssetsFreezer
impl pallet_assets_freezer::Config<KreivoAssetsInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeFreezeReason = RuntimeFreezeReason;
}

// #[runtime::pallet_index(13)]
// pub type Assets
parameter_types! {
	pub const AssetDeposit: Balance = UNITS / 10; // 1 / 10 UNITS deposit to create asset
	pub const AssetAccountDeposit: Balance = deposit(1, 16);
	pub const ApprovalDeposit: Balance = EXISTENTIAL_DEPOSIT;
	pub const AssetsStringLimit: u32 = 50;
	/// Key = 32 bytes, Value = 36 bytes (32+1+1+1+1)
	// https://github.com/paritytech/substrate/blob/069917b/frame/assets/src/lib.rs#L257L271
	pub const MetadataDepositBase: Balance = deposit(1, 68);
	pub const MetadataDepositPerByte: Balance = deposit(0, 1);
}

/// We allow root to execute privileged asset operations.
pub type AssetsForceOrigin = EnsureRoot<AccountId>;
pub type KreivoAssetsInstance = pallet_assets::Instance1;
pub type KreivoAssetsCall = pallet_assets::Call<Runtime, KreivoAssetsInstance>;

impl pallet_assets::Config<KreivoAssetsInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = FungibleAssetLocation;
	type AssetIdParameter = FungibleAssetLocation;
	type Currency = Balances;
	/// Only root can create assets and force state changes.
	type CreateOrigin = AsEnsureOriginWithArg<NeverEnsureOrigin<AccountId>>;
	type ForceOrigin = AssetsForceOrigin;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = AssetsStringLimit;
	type Freezer = AssetsFreezer;
	type Extra = ();
	type WeightInfo = weights::pallet_assets::WeightInfo<Runtime>;
	type CallbackHandle = ();
	type AssetAccountDeposit = AssetAccountDeposit;
	type RemoveItemsLimit = frame_support::traits::ConstU32<1000>;
	type MaxHolds = frame_support::traits::ConstU32<50>;
	type RuntimeHoldReason = RuntimeHoldReason;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

// #[runtime::pallet_index(14)]
// pub type AssetsTxPayment
impl pallet_asset_tx_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Fungibles = Assets;
	type OnChargeAssetTransaction = FungiblesAdapter<
		BalanceToAssetBalance<Balances, Runtime, ConvertInto, KreivoAssetsInstance>,
		AssetsToBlockAuthor<Runtime, KreivoAssetsInstance>,
	>;
}

// #[runtime::pallet_index(15)]
// pub type Vesting
parameter_types! {
	pub const MinVestedTransfer: Balance = 100 * CENTS;
	pub UnvestedFundsAllowedWithdrawReasons: WithdrawReasons =
		WithdrawReasons::except(WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE);
}

impl pallet_vesting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BlockNumberToBalance = ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = pallet_vesting::weights::SubstrateWeight<Runtime>;
	type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
	type BlockNumberProvider = System;
	const MAX_VESTING_SCHEDULES: u32 = 28;
}

// #[runtime::pallet_index(16)]
// pub type SkipFeeless
impl pallet_skip_feeless_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
}

// #[runtime::pallet_index(17)]
// pub type GasTxPayment

parameter_types! {
	pub MembershipIsNotExpired: Box<dyn SelectNonFungibleItem<CommunityId, MembershipId>> =
		Box::new(|community, membership| {
			use frame_support::traits::nonfungibles_v2::Inspect;
			let membership_expiration = b"membership_expiration";
			CommunityMemberships::typed_system_attribute(&community, Some(&membership), &membership_expiration)
				// If there's an expiration date, check it against block number
				.map(|expiration| System::block_number() <= expiration)
				// Otherwise, the membership will not expire
				.unwrap_or(true)
		});
}

pub type MembershipsGasTank =
	NonFungibleGasTank<Runtime, CommunityMemberships, pallet_nfts::ItemConfig, MembershipIsNotExpired>;

impl pallet_gas_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type GasBurner = MembershipsGasTank;
}