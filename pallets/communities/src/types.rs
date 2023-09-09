use frame_support::pallet_prelude::{Decode, Encode};
use frame_support::traits::fungibles::Inspect;
use frame_support::{sp_runtime::BoundedVec, traits::ConstU32};

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type CommunityId = u128;
pub type Cell = u32;
pub type AssetIdOf<T> = <<T as crate::Config>::Fungibles as Inspect<AccountIdOf<T>>>::AssetId;
pub type Field<const S: u32> = BoundedVec<u8, ConstU32<S>>;
pub type MemberListOf<T> = Vec<AccountIdOf<T>>;

#[derive(Encode, Decode)]
pub struct Community<T: crate::Config> {
	pub admin: AccountIdOf<T>,
	pub sufficient_asset_id: AssetIdOf<T>,
}

#[derive(Encode, Decode)]
pub struct CommunityMetadata {
	pub name: Field<64>,
	pub description: Field<256>,
	pub urls: BoundedVec<Field<32>, ConstU32<10>>,
	pub locations: BoundedVec<Cell, ConstU32<128>>,
}
