use sp_runtime::traits::CheckedConversion;

use super::*;

impl<T: Config> Pallet<T> {
	pub(crate) fn ensure_proposal_origin(
		community_id: &CommunityIdOf<T>,
		origin: PalletsOriginOf<T>,
	) -> DispatchResult {
		let community_account_id = Self::get_community_account_id(community_id);

		if let Some(o) = origin.clone().checked_into::<frame_system::Origin<T>>() {
			match o {
				frame_system::Origin::<T>::Signed(account) if account == community_account_id => Ok(()),
				_ => Err(Error::<T>::InvalidProposalOrigin.into()),
			}
		} else {
			match origin.checked_into::<pallet::Origin<T>>() {
				Some(_) => Ok(()),
				None => Err(Error::<T>::InvalidProposalOrigin.into()),
			}
		}
	}
}
