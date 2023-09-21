use super::*;
use frame_support::traits::tokens::fungibles::{roles::Inspect, Create, Destroy};

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
			community_assets_count == 0,
			min_balance,
		)?;

		<CommunityAssets<T>>::append(community_id, asset_id);

		Ok(())
	}
}
