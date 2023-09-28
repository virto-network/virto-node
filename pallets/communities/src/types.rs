use codec::MaxEncodedLen;
use frame_support::pallet_prelude::{Decode, Encode};
use frame_support::traits::{fungible::Inspect, fungibles::Inspect as InspectFuns};
use frame_support::{sp_runtime::BoundedVec, traits::ConstU32};
use scale_info::{prelude::vec::Vec, TypeInfo};
use sp_runtime::traits::StaticLookup;

use crate::Config;
use frame_system::Config as SystemConfig;

pub type AssetIdOf<T> = <<T as Config>::Assets as InspectFuns<AccountIdOf<T>>>::AssetId;
pub type BalanceOf<T> = <<T as Config>::Assets as InspectFuns<AccountIdOf<T>>>::Balance;
pub type NativeBalanceOf<T> = <<T as Config>::Balances as Inspect<AccountIdOf<T>>>::Balance;
pub type AccountIdOf<T> = <T as SystemConfig>::AccountId;
pub type AccountIdLookupOf<T> = <<T as SystemConfig>::Lookup as StaticLookup>::Source;
pub type CommunityIdOf<T> = <T as Config>::CommunityId;
pub type MembershipPassportOf<T> = <T as Config>::MembershipPassport;
pub type MemberListOf<T> = Vec<AccountIdOf<T>>;

pub type Cell = u32;

pub type SizedField<S> = BoundedVec<u8, S>;
pub type ConstSizedField<const S: u32> = SizedField<ConstU32<S>>;

/// The Community struct holds the basic definition of a community. It includes
/// the current state of a community, the [`AccountId`][1] for the community
/// admin, and (if any) the ID of the community-issued asset the community has
/// marked to be sufficient.
///
/// [1]: `frame_system::Config::AccountId`
#[derive(TypeInfo, Encode, Decode, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Community<T: crate::Config> {
	/// The [`AccountId`][1] of the current community administrator.
	///
	/// [1]: `frame_system::Config::AccountId`
	pub admin: AccountIdOf<T>,
	/// The current state of the community.
	pub state: CommunityState,
	/// The ID of the asset marked by the community as sufficient.
	pub sufficient_asset_id: Option<AssetIdOf<T>>,
}

/// The current state of the community. It represents whether a community
/// is awaiting to prove their contribution to the network, is active
/// and can operate, blocked due to a violation of network norms, or
/// it's being frozen by the community administrators.
#[derive(Default, TypeInfo, PartialEq, Encode, Decode, MaxEncodedLen)]
pub enum CommunityState {
	/// The community is currently awaiting to prove they are contributing
	/// to the network.
	#[default]
	Awaiting,
	/// The community has proven the required contributions to the network
	/// and can operate.
	Active,
	/// The community is frozen, and is temporality unable to operate. This
	/// state can be changed by thawing the community via a privileged call.
	Frozen,
	/// The community is blocked, typically as a result of a restriction imposed
	/// by violating the norms of the network. In practice, this is an
	/// equivalent to being `frozen`, howerver the state cannot be changed by
	/// the community administrators, but by submitting a proposal to the
	/// network for it to be changed.
	Blocked,
}

/// The CommunityMetadata struct stores some descriptive information about
/// the community.
#[derive(TypeInfo, Eq, PartialEq, Debug, Clone, Encode, Decode, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct CommunityMetadata<T: Config> {
	/// The name of the community
	pub name: ConstSizedField<64>,
	/// A short description of the community
	pub description: ConstSizedField<256>,
	/// A list of URLs that can lead to information about the community
	pub urls: BoundedVec<SizedField<T::MetadataUrlSize>, T::MaxUrls>,
	/// A list of physical locations where the community operates.
	pub locations: BoundedVec<Cell, T::MaxLocations>,
}

impl<T: Config> Default for CommunityMetadata<T> {
	fn default() -> Self {
		Self {
			name: Default::default(),
			description: Default::default(),
			urls: Default::default(),
			locations: Default::default(),
		}
	}
}
