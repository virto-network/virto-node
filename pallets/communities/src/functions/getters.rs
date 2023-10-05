use super::*;
use frame_support::sp_runtime::traits::AccountIdConversion;

impl<T: Config> Pallet<T> {
	pub(crate) fn get_community_account_id(community_id: &CommunityIdOf<T>) -> AccountIdOf<T> {
		T::PalletId::get().into_sub_account_truncating(community_id)
	}

	pub(crate) fn get_community_admin(community_id: &CommunityIdOf<T>) -> Result<AccountIdOf<T>, DispatchError> {
		let community = Self::community(community_id).ok_or(Error::<T>::CommunityDoesNotExist)?;

		Ok(community.admin)
	}
}