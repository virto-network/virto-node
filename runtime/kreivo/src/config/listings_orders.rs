use super::*;
use frame_support::traits::{fungible::HoldConsideration, EnsureOrigin, LinearStoragePrice, MapSuccess};

use pallet_listings::{InventoryId, InventoryIdFor, ItemIdOf};
use sp_runtime::traits::{AccountIdConversion, Verify};

#[cfg(not(feature = "runtime-benchmarks"))]
use frame_system::EnsureNever;
use frame_system::EnsureSigned;
use sp_core::ConstU128;
use sp_runtime::morph_types;
use virto_common::listings;

parameter_types! {
	pub KeyLimit: u32 = 64;
	pub ValueLimit: u32 = 256;
	/// How long before a charge is due it may first be attempted: 12 hours (0009-A11).
	pub const ListingsRenewalLead: BlockNumber = 12 * HOURS;
	/// The width of a bucket of the due queue: 10 minutes (0009-A6). At most `ListingsRenewalLead`.
	pub const ListingsDueBucketSize: BlockNumber = 10 * MINUTES;
	/// Provisional, until `F-08` measures them (`deployment/kreivo-0.18.md` §1).
	pub const ListingsMaxDuePerBucket: u32 = 64;
	/// Provisional, until `F-08` measures it.
	pub const ListingsMaxChargesPerBlock: u32 = 8;
	/// A TEST VALUE: the amount each subscription holds from its subscriber, in the native currency.
	/// `F-07b` sets the measured one (`m3-subscription-deposit`).
	pub const SubscriptionDepositAmount: Balance = CENTS;
	pub const SubscriptionHoldReason: RuntimeHoldReason =
		RuntimeHoldReason::Listings(pallet_listings::HoldReason::SubscriptionDeposit);
}

/// Anyone signed may subscribe to an inventory directly, except to merchant 0's, the collective's:
/// its subscriptions are not for a member to start (`DEC-8`).
pub struct EnsureSubscriber;

impl<Id> EnsureOriginWithArg<RuntimeOrigin, InventoryId<CommunityId, Id>> for EnsureSubscriber {
	type Success = AccountId;

	fn try_origin(
		o: RuntimeOrigin,
		InventoryId(merchant_id, _): &InventoryId<CommunityId, Id>,
	) -> Result<Self::Success, RuntimeOrigin> {
		if *merchant_id == 0 {
			return Err(o);
		}
		<EnsureSigned<AccountId> as EnsureOrigin<RuntimeOrigin>>::try_origin(o)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin(InventoryId(merchant_id, _): &InventoryId<CommunityId, Id>) -> Result<RuntimeOrigin, ()> {
		if *merchant_id == 0 {
			return Err(());
		}
		<EnsureSigned<AccountId> as EnsureOrigin<RuntimeOrigin>>::try_successful_origin()
	}
}

pub type ListingsInstance = pallet_listings::Instance1;

// #[runtime::pallet_index(61)]
// pub type Listings
pub struct EnsureCommunity;

impl<Id> EnsureOriginWithArg<RuntimeOrigin, InventoryId<CommunityId, Id>> for EnsureCommunity {
	type Success = AccountId;

	fn try_origin(
		o: RuntimeOrigin,
		InventoryId(community_id, _): &InventoryId<CommunityId, Id>,
	) -> Result<Self::Success, RuntimeOrigin> {
		match o.clone().caller {
			OriginCaller::Communities(origin) => (origin.id() == *community_id)
				.then_some(Communities::community_account(community_id))
				.ok_or(o),
			OriginCaller::system(frame_system::RawOrigin::Signed(ref who)) => {
				let Some((_, id)) = PalletId::try_from_sub_account::<CommunityId>(who) else {
					return Err(o);
				};
				ensure!(community_id == &id, o);
				Ok(who.clone())
			}
			_ => Err(o),
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin(InventoryId(community_id, _): &InventoryId<CommunityId, Id>) -> Result<RuntimeOrigin, ()> {
		Ok(RuntimeOrigin::signed(Communities::community_account(community_id)))
	}
}

impl pallet_listings::Config<ListingsInstance> for Runtime {
	type WeightInfo = weights::pallet_listings::WeightInfo<Self>;
	type CreateInventoryOrigin = EnsureCommunity;
	type InventoryAdminOrigin = EnsureCommunity;
	type MerchantId = CommunityId;
	type InventoryId = listings::InventoryId;
	type ItemSKU = listings::ItemId;
	type CollectionConfig =
		pallet_nfts::CollectionConfig<Balance, BlockNumberFor<Runtime>, InventoryIdFor<Runtime, ListingsInstance>>;
	type ItemConfig = pallet_nfts::ItemConfig;
	type Balances = Balances;
	type Assets = payments::NativeAndAssets;
	type Nonfungibles = ListingsCatalog;
	type NonfungiblesKeyLimit = KeyLimit;
	type NonfungiblesValueLimit = ValueLimit;
	type SubscribeOrigin = EnsureSubscriber;
	type Payments = Payments;
	type BlockNumberProvider = RelaychainData;
	type RenewalLead = ListingsRenewalLead;
	type DueBucketSize = ListingsDueBucketSize;
	type MaxDuePerBucket = ListingsMaxDuePerBucket;
	type MaxChargesPerBlock = ListingsMaxChargesPerBlock;
	type SubscriptionDeposit = HoldConsideration<
		AccountId,
		Balances,
		SubscriptionHoldReason,
		LinearStoragePrice<SubscriptionDepositAmount, ConstU128<0>, Balance>,
	>;
	type OnSubscriptionChanged = UsageSubscription;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = benchmarks::ListingsBenchmarkHelper;
}

// #[runtime::pallet_index(62)]
// pub type ListingsCatalog
impl pallet_nfts::Config<ListingsInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = InventoryIdFor<Self, ListingsInstance>;
	type ItemId = ItemIdOf<Self, ListingsInstance>;
	type Currency = Balances;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type ForceOrigin = EnsureNever<AccountId>;
	#[cfg(feature = "runtime-benchmarks")]
	type ForceOrigin = EnsureRoot<AccountId>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type CreateOrigin = EnsureNever<AccountId>;
	#[cfg(feature = "runtime-benchmarks")]
	type CreateOrigin = frame_system::EnsureSigned<AccountId>;
	type Locker = ();
	type CollectionDeposit = ();
	#[cfg(not(feature = "runtime-benchmarks"))]
	type ItemDeposit = ();
	#[cfg(feature = "runtime-benchmarks")]
	type ItemDeposit = ConstU128<UNITS>;
	type MetadataDepositBase = ();
	type AttributeDepositBase = ();
	type DepositPerByte = ();
	type StringLimit = ValueLimit;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type ApprovalsLimit = ();
	#[cfg(feature = "runtime-benchmarks")]
	type ApprovalsLimit = ConstU32<1>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type ItemAttributesApprovalsLimit = ();
	#[cfg(feature = "runtime-benchmarks")]
	type ItemAttributesApprovalsLimit = ConstU32<1>;
	// Tips, swap deadlines and multi-attribute calls are off on the catalog (a zero bound rejects any
	// real call). Benchmark builds need a range to fit a slope, so they get small bounds instead;
	// the weights still apply, since a call that exceeds the live zero bound fails anyway.
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MaxTips = ();
	#[cfg(feature = "runtime-benchmarks")]
	type MaxTips = ConstU32<10>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MaxDeadlineDuration = ();
	#[cfg(feature = "runtime-benchmarks")]
	type MaxDeadlineDuration = ConstU32<{ 30 * runtime_constants::time::DAYS }>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MaxAttributesPerCall = ();
	#[cfg(feature = "runtime-benchmarks")]
	type MaxAttributesPerCall = ConstU32<10>;
	type Features = ();
	type OffchainSignature = Signature;
	type OffchainPublic = <Signature as Verify>::Signer;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = benchmarks::ListingsCatalogBenchmarkHelper;
	type WeightInfo = weights::pallet_nfts_listings_catalog::WeightInfo<Self>;
	type BlockNumberProvider = RelaychainData;
}

morph_types! {
	pub type MaxCartsForRegularUsers = |id: AccountId| -> (AccountId, u32) { (id, 5) };
	pub type MaxItemsForRegularUsers = |id: AccountId| -> (AccountId, u32) { (id, 64) };
}

parameter_types! {
	pub MaxLifetimeForCheckoutOrder: BlockNumber = 30 * MINUTES;
	pub MaxCartLen: u32 = 5;
	pub MaxItemLen: u32 = 64;
}

impl pallet_orders::Config<ListingsInstance> for Runtime {
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = weights::pallet_orders::WeightInfo<Self>;
	type CreateOrigin = MapSuccess<EnsureSigned<AccountId>, MaxCartsForRegularUsers>;
	type OrderAdminOrigin = MapSuccess<EnsureSigned<AccountId>, MaxItemsForRegularUsers>;
	type PaymentOrigin = EnsureSigned<AccountId>;
	type OrderId = u64;
	type Listings = Listings;
	type Payments = Payments;
	type Scheduler = Scheduler;
	type BlockNumberProvider = RelaychainData;
	type MaxLifetimeForCheckoutOrder = MaxLifetimeForCheckoutOrder;
	type MaxCartLen = MaxCartLen;
	type MaxItemLen = MaxItemLen;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = benchmarks::OrdersBenchmarkHelper;
}

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks {
	use super::*;
	use pallet_orders::InventoryIdOf;

	pub struct ListingsBenchmarkHelper;

	impl pallet_listings::BenchmarkHelper<InventoryIdFor<Runtime, ListingsInstance>> for ListingsBenchmarkHelper {
		fn inventory_id() -> InventoryIdFor<Runtime, ListingsInstance> {
			// Not merchant 0, the collective's: `EnsureSubscriber` rejects subscribing to its
			// inventories, which made every subscriber benchmark skip as weightless.
			InventoryId(1, 1)
		}
	}

	impl pallet_listings::SubscriptionsBenchmarkHelper<AccountId, FungibleAssetLocation, Balance>
		for ListingsBenchmarkHelper
	{
		fn price() -> (FungibleAssetLocation, Balance) {
			use frame_support::traits::fungibles::{Create, Inspect};
			let asset = FungibleAssetLocation::Here(1);
			if !Assets::asset_exists(asset) {
				<Assets as Create<_>>::create(asset, TreasuryAccount::get(), true, EXISTENTIAL_DEPOSIT)
					.expect("creating an asset class is expected to succeed; qed");
			}
			(asset, 100 * EXISTENTIAL_DEPOSIT)
		}

		fn fund(who: &AccountId, asset: &FungibleAssetLocation, amount: Balance) {
			use frame_support::traits::fungibles::Mutate;
			Assets::mint_into(*asset, who, amount.saturating_mul(1_000)).expect("minting is expected to succeed; qed");
		}
	}

	pub struct ListingsCatalogBenchmarkHelper;

	impl
		pallet_nfts::BenchmarkHelper<
			InventoryIdFor<Runtime, ListingsInstance>,
			ItemIdOf<Runtime, ListingsInstance>,
			sp_runtime::MultiSigner,
			sp_runtime::AccountId32,
			sp_runtime::MultiSignature,
		> for ListingsCatalogBenchmarkHelper
	{
		fn collection(i: u16) -> InventoryIdFor<Runtime, ListingsInstance> {
			InventoryId(0, i.into())
		}

		fn item(i: u16) -> ItemIdOf<Runtime, ListingsInstance> {
			i.into()
		}

		fn signer() -> (sp_runtime::MultiSigner, sp_runtime::AccountId32) {
			<() as pallet_nfts::BenchmarkHelper<
				u16,
				u16,
				sp_runtime::MultiSigner,
				sp_runtime::AccountId32,
				sp_runtime::MultiSignature,
			>>::signer()
		}

		fn sign(signer: &sp_runtime::MultiSigner, message: &[u8]) -> sp_runtime::MultiSignature {
			<() as pallet_nfts::BenchmarkHelper<
				u16,
				u16,
				sp_runtime::MultiSigner,
				sp_runtime::AccountId32,
				sp_runtime::MultiSignature,
			>>::sign(signer, message)
		}
	}

	type MerchantIdOf<T, I> = <T as pallet_listings::Config<I>>::MerchantId;

	pub struct OrdersBenchmarkHelper;

	impl pallet_orders::BenchmarkHelper<Runtime, ListingsInstance> for OrdersBenchmarkHelper {
		type Balances = Balances;
		type Assets = Assets;
		type InventoryDeposit = <Runtime as pallet_nfts::Config<ListingsInstance>>::CollectionDeposit;
		type ItemDeposit = <Runtime as pallet_nfts::Config<ListingsInstance>>::ItemDeposit;

		fn inventory_id() -> (
			MerchantIdOf<Runtime, ListingsInstance>,
			InventoryIdOf<Runtime, ListingsInstance>,
		) {
			(0, 0)
		}

		fn item_id(i: usize) -> ItemIdOf<Runtime, ListingsInstance> {
			i as u64
		}
	}
}
