use super::*;

mod indices;

use frame_support::traits::fungible;
use frame_support::traits::EitherOf;
use frame_system::EnsureSigned;
use pallet_communities::origin::AsSignedByCommunity;
use sp_runtime::traits::{AccountIdConversion, Convert};
use sp_runtime::Either;

pub use indices::pallet_payment_indices;

/// Tells the native token from the assets: `Here(0)` is `Balances`, any other id is an `Assets` asset
/// (`DEC-11`). Never create `Assets` asset `Here(0)` (`RISK-5`).
pub struct NativeOrAsset;

impl Convert<FungibleAssetLocation, Either<(), FungibleAssetLocation>> for NativeOrAsset {
	fn convert(asset: FungibleAssetLocation) -> Either<(), FungibleAssetLocation> {
		match asset {
			// The benchmarks of payments and orders pick the default asset id, `Here(0)`, and mint
			// amounts far below the native existential deposit, so benchmark builds keep it in
			// `Assets`, where the benchmarks create it.
			#[cfg(not(feature = "runtime-benchmarks"))]
			FungibleAssetLocation::Here(0) => Either::Left(()),
			asset => Either::Right(asset),
		}
	}
}

/// What payments and listings see: KSM as `Here(0)`, and every asset of `Assets`.
pub type NativeAndAssets = fungible::UnionOf<Balances, Assets, NativeOrAsset, FungibleAssetLocation, AccountId>;
/// The holds of [`NativeAndAssets`].
pub type NativeAndAssetsHold =
	fungible::UnionOf<Balances, AssetsHolder, NativeOrAsset, FungibleAssetLocation, AccountId>;

parameter_types! {
	pub const MaxRemarkLength: u8 = 50;
	pub const IncentivePercentage: Percent = Percent::from_percent(INCENTIVE_PERCENTAGE);
	pub const PaymentPalletId: PalletId = PalletId(*b"payments");
}

pub struct KreivoFeeHandler;

const MANDATORY_FEE: bool = true;
pub const SENDER_FEE: Percent = Percent::from_percent(1);
pub const BENEFICIARY_FEE: Percent = Percent::from_percent(3);
pub const INCENTIVE_PERCENTAGE: u8 = 10;

impl FeeHandler<Runtime> for KreivoFeeHandler {
	fn apply_fees(
		asset: &AssetIdOf<Runtime>,
		sender: &AccountId,
		beneficiary: &AccountId,
		amount: &Balance,
		_remark: Option<&[u8]>,
	) -> Fees<Runtime> {
		// A charge to the payee (the Treasury) pays no fee on either side (`DEC-5`, `REQ-BL-3`).
		if beneficiary == &TreasuryAccount::get() {
			return Fees {
				sender_pays: Default::default(),
				beneficiary_pays: Default::default(),
			};
		}
		let min = <NativeAndAssets as fungibles::Inspect<AccountId>>::minimum_balance(*asset);
		let pallet_id = communities::CommunityPalletId::get();
		let default_fee = |fee: Percent| (TreasuryAccount::get(), min.max(fee.mul_floor(*amount)), MANDATORY_FEE);
		let is_community =
			|who| matches!(PalletId::try_from_sub_account::<CommunityId>(who), Some((pid, _)) if pallet_id == pid );

		let mut sender_fees = vec![];
		let mut beneficiary_fees = vec![];

		if !is_community(sender) {
			sender_fees.push(default_fee(SENDER_FEE))
		}
		if !is_community(beneficiary) {
			beneficiary_fees.push(default_fee(BENEFICIARY_FEE))
		}
		Fees {
			sender_pays: BoundedVec::try_from(sender_fees).unwrap(),
			beneficiary_pays: BoundedVec::try_from(beneficiary_fees).unwrap(),
		}
	}
}

impl pallet_payment_indices::Config for Runtime {}

impl pallet_payments::Config for Runtime {
	type PalletsOrigin = OriginCaller;
	type RuntimeHoldReason = RuntimeHoldReason;
	type WeightInfo = weights::pallet_payments::WeightInfo<Self>;
	type SenderOrigin = EitherOf<AsSignedByCommunity<Self>, EnsureSigned<AccountId>>;
	type BeneficiaryOrigin = EnsureSigned<AccountId>;
	type DisputeResolver = frame_system::EnsureRootWithSuccess<AccountId, TreasuryAccount>;
	type PaymentId = virto_common::PaymentId;
	type Assets = NativeAndAssets;
	type AssetsHold = NativeAndAssetsHold;
	type BlockNumberProvider = RelaychainData;
	type FeeHandler = KreivoFeeHandler;
	type Scheduler = Scheduler;
	type Preimages = Preimage;
	type OnPaymentStatusChanged = Orders;
	type GeneratePaymentId = PaymentIndices;
	type PalletId = PaymentPalletId;
	type IncentivePercentage = IncentivePercentage;
	type MaxRemarkLength = MaxRemarkLength;
	type MaxFees = ConstU32<50>;
	type MaxDiscounts = ConstU32<10>;
	type CancelBufferBlockLength = ConstU32<{ 2 * DAYS }>;
}
