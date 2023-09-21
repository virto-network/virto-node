use super::*;
use frame_support::traits::tokens::fungibles::{roles::Inspect, Create, Destroy};
use sp_runtime::traits::Zero;

impl<T: Config> Pallet<T> {
	pub(crate) fn do_create_asset(
		community_id: &CommunityIdOf<T>,
		asset_id: AssetIdOf<T>,
		min_balance: BalanceOf<T>,
	) -> DispatchResult {
		let community_account_id = Self::get_community_account_id(community_id);
		let community_assets_count = <CommunityAssets<T>>::decode_len(community_id).unwrap_or_default();

		T::Assets::create(
			asset_id.clone(),
			community_account_id,
			community_assets_count.is_zero(),
			min_balance,
		)?;

		<CommunityAssets<T>>::append(community_id, asset_id);

		Ok(())
	}

	pub(crate) fn do_destroy_asset(community_id: &CommunityIdOf<T>, asset_id: AssetIdOf<T>) -> DispatchResult {
		let community_account_id = Self::get_community_account_id(community_id);

		let asset_owner = T::Assets::owner(asset_id.clone()).ok_or(Error::<T>::UnknownAsset)?;

		if !<CommunityAssets<T>>::get(community_id).contains(&asset_id) || asset_owner != community_account_id {
			return Err(Error::<T>::CannotDestroyUncontrolledAsset)?;
		}

		T::Assets::start_destroy(asset_id.clone(), Some(community_account_id))?;
		T::Assets::destroy_accounts(asset_id.clone(), u32::MAX)?;
		T::Assets::destroy_approvals(asset_id.clone(), u32::MAX)?;
		T::Assets::finish_destroy(asset_id)?;

		Ok(())
	}
}
