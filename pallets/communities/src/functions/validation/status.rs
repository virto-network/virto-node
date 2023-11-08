use super::*;

impl<T: Config> Pallet<T> {
	pub(crate) fn ensure_active(community_id: &CommunityIdOf<T>) -> DispatchResult {
		match Self::community(community_id)
			.map(|info| info.state)
			.ok_or(Error::<T>::CommunityDoesNotExist)?
		{
			CommunityState::Active => Ok(()),
			_ => Err(Error::<T>::CommunityNotActive)?,
		}
	}
}
