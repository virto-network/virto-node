use super::*;

impl<T: Config> Pallet<T> {
	pub(crate) fn ensure_privileged(origin: OriginFor<T>, community_id: &T::CommunityId) -> Result<(), DispatchError> {
		if let Some(caller) = ensure_signed_or_root(origin)? {
			if caller != Self::get_community_admin(community_id)? {
				return Err(DispatchError::BadOrigin);
			}
		}

		Ok(())
	}

	pub(crate) fn do_insert_member(community_id: &T::CommunityId, who: &T::AccountId) -> DispatchResult {
		if <CommunityMembers<T>>::contains_key(community_id, who) {
			return Err(Error::<T>::AlreadyAMember.into());
		}

		<CommunityMembers<T>>::insert::<T::CommunityId, T::AccountId, T::MemberRank>(
			community_id.clone(),
			who.clone(),
			Default::default(),
		);

		let members_count = <CommunityMembersCount<T>>::try_get(community_id).unwrap_or_default();
		<CommunityMembersCount<T>>::set(community_id, members_count.checked_add(1));

		Ok(())
	}
}
