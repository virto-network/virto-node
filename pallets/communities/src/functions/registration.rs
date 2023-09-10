use super::*;

use frame_support::traits::tokens::fungible;

impl<T: Config> Pallet<T> {
	pub(crate) fn community_exists(community_id: &T::CommunityId) -> bool {
		<CommunityInfo<T>>::contains_key(community_id) && <CommunityMembersCount<T>>::contains_key(community_id)
	}

	/// Stores an initial info about the community
	/// Sets the caller as the community admin, the initial community state
	/// as
	pub(crate) fn do_register_community(who: &T::AccountId, community_id: &T::CommunityId) -> DispatchResult {
		// Check that the community doesn't exist
		if Self::community_exists(&community_id) {
			return Err(Error::<T>::CommunityAlreadyExists.into());
		}

		<CommunityInfo<T>>::insert(
			community_id.clone(),
			Community {
				admin: who.clone(),
				state: Default::default(),
				sufficient_asset_id: Default::default(),
			},
		);

		Self::do_insert_member(community_id, who)?;

		Ok(())
	}

	/// Takes a deposit from the caller and
	pub(crate) fn do_create_community_account(
		caller: &AccountIdOf<T>,
		community_id: &CommunityIdOf<T>,
	) -> DispatchResult {
		let community_account_id = Self::get_community_account_id(community_id);
		let minimum_balance = <T::Balances as fungible::Inspect<T::AccountId>>::minimum_balance();

		<T::Balances as fungible::Mutate<T::AccountId>>::transfer(
			&caller,
			&community_account_id,
			minimum_balance,
			frame_support::traits::tokens::Preservation::Preserve,
		)?;

		// Lock funds so the account can exist at all times
		<T::Balances as fungible::MutateFreeze<T::AccountId>>::set_freeze(
			&T::FreezeIdentifier::get(),
			&community_account_id,
			minimum_balance,
		)?;

		Ok(())
	}

	pub(crate) fn do_set_metadata(
		community_id: &CommunityIdOf<T>,
		value: types::CommunityMetadata<T>,
	) -> DispatchResult {
		<pallet::CommunityMetadata<T>>::try_mutate(community_id, |metadata| {
			if let Some(metadata) = metadata {
				metadata.name = value.name;
				metadata.description = value.description;
				metadata.urls = value.urls;
				metadata.locations = value.locations;
			} else {
				*metadata = Some(value);
			}

			Ok::<(), DispatchError>(())
		})?;

		Ok(())
	}
}
