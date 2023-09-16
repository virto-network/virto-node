use super::*;

impl<T: Config> Pallet<T> {
	pub(crate) fn ensure_active(community_id: &T::CommunityId) -> DispatchResult {
		let community_info =
			<CommunityInfo<T>>::try_get(community_id).map_err(|_| Error::<T>::CommunityDoesNotExist)?;

		if community_info.state != CommunityState::Active {
			Err(Error::<T>::CommunityNotActive)?
		}

		Ok(())
	}

	pub(crate) fn do_force_complete_challenge(community_id: &T::CommunityId) -> DispatchResult {
		<CommunityInfo<T>>::try_mutate_exists(community_id, |value| {
			let Some(community_info) = value else {
        return Err::<(), DispatchError>(Error::<T>::CommunityDoesNotExist.into());
      };

			community_info.state = CommunityState::Active;

			Ok(())
		})
	}
}
