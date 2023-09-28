use super::*;

impl<T: Config> Pallet<T> {
	pub(crate) fn ensure_origin_member(
		origin: OriginFor<T>,
		community_id: &CommunityIdOf<T>,
	) -> Result<(), DispatchError> {
		let caller = ensure_signed(origin)?;

		if Self::member_information(community_id, caller).is_none() {
			return Err(DispatchError::BadOrigin);
		}

		Ok(())
	}

	pub(crate) fn ensure_origin_privileged(
		origin: OriginFor<T>,
		community_id: &CommunityIdOf<T>,
	) -> Result<(), DispatchError> {
		if let Some(caller) = ensure_signed_or_root(origin)? {
			if caller != Self::get_community_admin(community_id)? {
				return Err(DispatchError::BadOrigin);
			}
		}

		Ok(())
	}

	pub(crate) fn do_insert_member(community_id: &CommunityIdOf<T>, who: &AccountIdOf<T>) -> DispatchResult {
		<CommunityMembers<T>>::try_mutate_exists(community_id, who, |value| {
			if value.is_some() {
				return Err(Error::<T>::AlreadyAMember.into());
			}

			// Inserts the member
			*value = Some(Default::default());

			// Increases member count
			let members_count = Self::members_count(community_id).unwrap_or_default();
			<CommunityMembersCount<T>>::set(community_id, members_count.checked_add(1));

			Ok(())
		})
	}

	pub(crate) fn do_remove_member(community_id: &T::CommunityId, who: &T::AccountId) -> DispatchResult {
		<CommunityMembers<T>>::try_mutate_exists(community_id, who, |value| {
			if value.is_none() {
				return Err(Error::<T>::NotAMember.into());
			}

			let Some(community_info) = Self::community(community_id) else {
				return Err(Error::<T>::CommunityDoesNotExist.into());
			};

			if community_info.admin == *who {
				return Err(Error::<T>::CannotRemoveAdmin.into());
			}

			// Removes the member
			*value = None;

			// Decreases member count
			let members_count = Self::members_count(community_id).unwrap_or_default();
			<CommunityMembersCount<T>>::set(community_id, members_count.checked_sub(1));

			Ok(())
		})
	}
}
