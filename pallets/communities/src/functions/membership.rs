use super::*;
use crate::traits::member_rank::Mutate;

impl<T: Config> Pallet<T> {
	pub(crate) fn ensure_origin_member(
		origin: OriginFor<T>,
		community_id: &CommunityIdOf<T>,
	) -> Result<AccountIdOf<T>, DispatchError> {
		let caller = ensure_signed(origin)?;

		Self::membership(community_id, &caller)
			.ok_or(DispatchError::BadOrigin)
			.map(|_| caller)
	}

	#[allow(dead_code)]
	pub(crate) fn ensure_member(
		community_id: &CommunityIdOf<T>,
		who: &AccountIdOf<T>,
	) -> Result<MembershipOf<T>, DispatchError> {
		Self::membership(community_id, who).ok_or(Error::<T>::NotAMember.into())
	}

	pub(crate) fn ensure_origin_privileged(
		origin: OriginFor<T>,
		community_id: &CommunityIdOf<T>,
	) -> Result<Option<AccountIdOf<T>>, DispatchError> {
		if let Some(caller) = ensure_signed_or_root(origin)? {
			if let Some(admin) = Self::get_community_admin(community_id) && admin == caller {
				return Ok(Some(admin))
			} else {
				return Err(DispatchError::BadOrigin)
			}
		}

		Ok(None)
	}

	/// Inserts `who` into the community
	pub(crate) fn do_insert_member(community_id: &CommunityIdOf<T>, who: &AccountIdOf<T>) -> DispatchResult {
		Members::<T>::try_mutate_exists(community_id, who, |value| {
			if value.is_some() {
				return Err(Error::<T>::AlreadyAMember.into());
			}

			// Inserts the member
			*value = Some(Default::default());
			MemberRanks::<T>::set(community_id, who, Some(Default::default()));

			// Increases member count
			let members_count = Self::members_count(community_id).unwrap_or_default();
			MembersCount::<T>::set(community_id, members_count.checked_add(1));

			Ok(())
		})
	}

	pub(crate) fn do_promote_member(community_id: &CommunityIdOf<T>, who: &AccountIdOf<T>) -> DispatchResult {
		MemberRanks::<T>::try_mutate(community_id, who, |maybe_rank| {
			let Some(rank) = maybe_rank else {
				return Err(Error::<T>::NotAMember)?;
			};

			*maybe_rank = rank.promote();

			if maybe_rank.is_none() {
				return Err(Error::<T>::ExceededPromoteBound)?;
			}

			Ok(())
		})
	}

	pub(crate) fn do_demote_member(community_id: &CommunityIdOf<T>, who: &AccountIdOf<T>) -> DispatchResult {
		MemberRanks::<T>::try_mutate(community_id, who, |maybe_rank| {
			let Some(rank) = maybe_rank else {
				return Err(Error::<T>::NotAMember)?;
			};

			*maybe_rank = rank.demote();

			if maybe_rank.is_none() {
				return Err(Error::<T>::ExceededDemoteBound)?;
			}

			Ok(())
		})
	}

	pub(crate) fn do_remove_member(community_id: &T::CommunityId, who: &T::AccountId) -> DispatchResult {
		Members::<T>::try_mutate_exists(community_id, who, |value| {
			if value.is_none() {
				return Err(Error::<T>::NotAMember.into());
			}

			if let Some(community_admin) = Self::get_community_admin(community_id) && community_admin == *who {
				return Err(Error::<T>::CannotRemoveAdmin.into());
			}

			// Removes the member
			*value = None;

			// Decreases member count
			let members_count = Self::members_count(community_id).unwrap_or_default();
			MembersCount::<T>::set(community_id, members_count.checked_sub(1));

			Ok(())
		})
	}
}
