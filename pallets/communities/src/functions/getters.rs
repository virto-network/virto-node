use super::*;
use frame_support::sp_runtime::traits::AccountIdConversion;

impl<T: Config> Pallet<T> {
	pub(crate) fn get_community_account_id(community_id: &T::CommunityId) -> T::AccountId {
		T::PalletId::get().into_sub_account_truncating(community_id)
	}
}
