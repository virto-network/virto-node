use codec::MaxEncodedLen;
use frame_support::pallet_prelude::{Decode, Encode};
use frame_support::traits::fungibles::Inspect;
use frame_support::{sp_runtime::BoundedVec, traits::ConstU32};
use scale_info::TypeInfo;

use crate::Config;
use frame_system::Config as SystemConfig;

pub type AccountIdOf<T> = <T as SystemConfig>::AccountId;
pub type CommunityIdOf<T> = <T as Config>::CommunityId;
pub type MembershipPassportOf<T> = <T as Config>::MembershipPassport;
pub type AssetIdOf<T> = <<T as Config>::Assets as Inspect<AccountIdOf<T>>>::AssetId;
pub type MemberListOf<T> = Vec<AccountIdOf<T>>;

pub type Cell = u32;

pub type Field<const S: u32> = BoundedVec<u8, ConstU32<S>>;

#[derive(TypeInfo, Encode, Decode, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Community<T: crate::Config> {
	pub admin: AccountIdOf<T>,
	pub state: CommunityState,
	pub sufficient_asset_id: Option<AssetIdOf<T>>,
}

#[derive(TypeInfo, Encode, Decode, MaxEncodedLen)]
pub enum CommunityState {
	Awaiting,
	Active,
	Frozen,
	Blocked,
}

#[derive(TypeInfo, Eq, PartialEq, Debug, Clone, Encode, Decode, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct CommunityMetadata<T: Config> {
	pub(super) name: Field<64>,
	pub(super) description: Field<256>,
	pub(super) urls: BoundedVec<BoundedVec<u8, T::MetadataUrlSize>, T::MaxUrls>,
	pub(super) locations: BoundedVec<Cell, T::MaxLocations>,
}

impl Default for CommunityState {
	fn default() -> Self {
		CommunityState::Awaiting
	}
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
