use super::*;
use frame_support::traits::tokens::Preservation;

impl<T: Config> Pallet<T> {
	pub(crate) fn do_assets_transfer(
		community_id: &CommunityIdOf<T>,
		asset_id: AssetIdOf<T>,
		dest: &AccountIdOf<T>,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		let community_account_id = Self::get_community_account_id(community_id);
		T::Assets::transfer(asset_id, &community_account_id, dest, amount, Preservation::Preserve)?;

		Ok(())
	}

	pub(crate) fn do_balance_transfer(
		community_id: &CommunityIdOf<T>,
		dest: &AccountIdOf<T>,
		amount: NativeBalanceOf<T>,
	) -> DispatchResult {
		let community_account_id = Self::get_community_account_id(community_id);
		T::Balances::transfer(&community_account_id, dest, amount, Preservation::Preserve)?;

		Ok(())
	}
}
