use frame_contrib_traits::listings;
use frame_contrib_traits::listings::ListingsIdentifier;
use frame_contrib_traits::memberships;
use frame_support::traits::{fungible, fungibles, Contains, Get};
use frame_support::weights::Weight;
use frame_support::Parameter;
use parity_scale_codec::MaxEncodedLen;

pub trait Config: pallet_contracts::Config
where
	MembershipOf<Self>: Parameter + MaxEncodedLen,
{
	/// A type that implements the balances APIs.
	type Balances: fungible::Inspect<Self::AccountId>;
	/// A type that implements the assets' APIs.
	type Assets: fungibles::Inspect<Self::AccountId>
		+ fungibles::Mutate<Self::AccountId>
		+ fungibles::roles::Inspect<Self::AccountId>;
	/// The assets contracts may mint and burn. Any other asset is refused,
	/// whatever role the contract's group holds on it.
	type MintableAssets: Contains<AssetIdOf<Self>>;
	/// The weight charged for [`AssetsAPI::mint`][crate::apis::AssetsAPI::mint],
	/// before doing any work.
	///
	/// It must cover minting the asset, plus the reads to find the contract's
	/// group and the asset's roles.
	type MintWeight: Get<Weight>;
	/// The weight charged for [`AssetsAPI::burn`][crate::apis::AssetsAPI::burn],
	/// before doing any work.
	///
	/// It must cover burning the asset, plus the reads to find the contract's
	/// group and the asset's roles, and the reads to check the asset and the
	/// account can be burnt from.
	type BurnWeight: Get<Weight>;
	/// A type that implements the `MerchantIdInfo` trait.
	type MerchantIdInfo: MerchantIdInfo<AccountIdOf<Self>>;
	/// A type that implements the listings' APIs.
	type Listings: listings::InspectInventory<
			InventoryId = <Self::Listings as listings::InspectItem<AccountIdOf<Self>>>::InventoryId,
		> + listings::InventoryLifecycle<Self::AccountId, MerchantId = MerchantIdOf<Self>>
		+ listings::MutateInventory<MerchantId = MerchantIdOf<Self>>
		+ listings::InspectItem<
			Self::AccountId,
			MerchantId = MerchantIdOf<Self>,
			Asset = AssetIdOf<Self>,
			Balance = AssetBalanceOf<Self>,
		> + listings::MutateItem<
			Self::AccountId,
			MerchantId = MerchantIdOf<Self>,
			Asset = AssetIdOf<Self>,
			Balance = AssetBalanceOf<Self>,
		>;
	/// A type that implements the `GroupInfo` trait.
	type GroupInfo: GroupInfo<AccountIdOf<Self>, Group = GroupOf<Self>>;
	/// A type that implements the memberships' APIs.
	type Memberships: memberships::Inspect<Self::AccountId>
		+ memberships::InspectEnumerable<Self::AccountId>
		+ memberships::Attributes<Self::AccountId>
		+ memberships::Manager<Self::AccountId>
		+ memberships::Rank<Self::AccountId>;
}

pub trait MerchantIdInfo<AccountId> {
	type MerchantId: ListingsIdentifier;

	fn maybe_merchant_id(who: &AccountId) -> Option<Self::MerchantId>;
}

pub trait GroupInfo<AccountId> {
	type Group: Parameter;

	/// The group a contract account belongs to, if any.
	fn maybe_group(who: &AccountId) -> Option<Self::Group>;

	/// The account that acts on behalf of a `group`.
	fn group_account(group: &Self::Group) -> AccountId;
}

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type MerchantIdOf<T> = <<T as Config>::MerchantIdInfo as MerchantIdInfo<AccountIdOf<T>>>::MerchantId;
pub type AssetIdOf<T> = <<T as Config>::Assets as fungibles::Inspect<AccountIdOf<T>>>::AssetId;
pub type AssetBalanceOf<T> = <<T as Config>::Assets as fungibles::Inspect<AccountIdOf<T>>>::Balance;
pub type GroupOf<T> = <<T as Config>::Memberships as memberships::Inspect<AccountIdOf<T>>>::Group;
pub type MembershipOf<T> = <<T as Config>::Memberships as memberships::Inspect<AccountIdOf<T>>>::Membership;
