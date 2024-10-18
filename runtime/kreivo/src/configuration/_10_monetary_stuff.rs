use super::*;

use fc_traits_gas_tank::NonFungibleGasBurner;

use pallet_asset_tx_payment::FungiblesAdapter;
use pallet_assets::BalanceToAssetBalance;
use pallet_transaction_payment::FungibleAdapter;
use runtime_common::impls::AssetsToBlockAuthor;

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = FungibleAdapter<Balances, ResolveTo<TreasuryAccount, Balances>>;
	type WeightToFee = WeightToFee;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type OperationalFeeMultiplier = ConstU8<5>;
}

impl pallet_asset_tx_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Fungibles = Assets;
	type OnChargeAssetTransaction = FungiblesAdapter<
		BalanceToAssetBalance<Balances, Runtime, ConvertInto, KreivoAssetsInstance>,
		AssetsToBlockAuthor<Runtime, KreivoAssetsInstance>,
	>;
}

impl pallet_gas_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type GasBurner = NonFungibleGasBurner<Runtime, CommunityMemberships, pallet_nfts::ItemConfig>;
}
