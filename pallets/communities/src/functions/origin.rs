use super::*;

macro_rules! as_origin {
	($origin: ident, $t: ty) => {{
		TryInto::<$t>::try_into($origin.clone()).ok()
	}};
}

impl<T: Config> Pallet<T> {
	pub(crate) fn ensure_proposal_origin(
		community_id: &CommunityIdOf<T>,
		origin: PalletsOriginOf<T>,
	) -> DispatchResult {
		let community_account_id = Self::get_community_account_id(community_id);

		if let Some(o) = as_origin!(origin, frame_system::Origin<T>) {
			match o {
				frame_system::Origin::<T>::Signed(account) if account == community_account_id => Ok(()),
				_ => Err(Error::<T>::InvalidProposalOrigin.into()),
			}
		} else {
			match as_origin!(origin, pallet::Origin<T>) {
				Some(_) => Ok(()),
				None => Err(Error::<T>::InvalidProposalOrigin.into()),
			}
		}
	}
}
