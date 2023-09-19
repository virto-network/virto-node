use super::*;

impl<T: Config> Pallet<T> {
	pub(crate) fn ensure_active(community_id: &CommunityIdOf<T>) -> DispatchResult {
		let community_info = Self::community(community_id).ok_or(Error::<T>::CommunityDoesNotExist)?;

		if community_info.state != CommunityState::Active {
			Err(Error::<T>::CommunityNotActive)?
		}

		Ok(())
	}

	#[allow(dead_code)]
	pub(crate) fn do_force_complete_challenge(community_id: &CommunityIdOf<T>) -> DispatchResult {
		Info::<T>::try_mutate_exists(community_id, |maybe_info| {
			let Some(info) = maybe_info else {
				return Err::<(), DispatchError>(Error::<T>::CommunityDoesNotExist.into());
			};

			info.state = CommunityState::Active;

			Ok(())
		})
	}
}
