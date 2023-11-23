use crate::{
	types::{AccountIdOf, CommunityGovernanceStrategy, CommunityIdOf},
	Config, GovernanceStrategy, Pallet,
};
use frame_support::sp_runtime::traits::{AccountIdConversion, Get};

impl<T: Config> Pallet<T> {
	pub(crate) fn get_community_account_id(community_id: &CommunityIdOf<T>) -> AccountIdOf<T> {
		T::PalletId::get().into_sub_account_truncating(community_id)
	}

	pub(crate) fn get_community_admin(community_id: &CommunityIdOf<T>) -> Option<AccountIdOf<T>> {
		match GovernanceStrategy::<T>::get(community_id) {
			Some(CommunityGovernanceStrategy::AdminBased(admin)) => Some(admin),
			_ => None,
		}
	}
}
