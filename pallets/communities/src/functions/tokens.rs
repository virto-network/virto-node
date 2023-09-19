use super::*;

impl<T: Config> Pallet<T> {
	pub(crate) fn do_assets_transfer(
		community_id: &CommunityIdOf<T>,
		asset_id: AssetIdOf<T>,
		dest: &AccountIdOf<T>,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		let community_account_id = Self::get_community_account_id(&community_id);
		T::Assets::transfer(
			asset_id,
			&community_account_id,
			dest,
			amount,
			frame_support::traits::tokens::Preservation::Preserve,
		)?;

		Ok(())
	}
}
