use crate::{
	types::{
		AccountIdOf, CommunityIdOf, CommunityInfo, CommunityMetadata, CommunityState, ConstSizedField, MembershipId,
		PalletsOriginOf,
	},
	CommunityIdFor, Config, Error, Info, Metadata, Pallet,
};
use frame_support::{
	pallet_prelude::*,
	traits::{GenericRank, MembershipInspect, RankedMembership},
};
use sp_runtime::traits::AccountIdConversion;
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
	#[inline]
	pub fn community_account(community_id: &T::CommunityId) -> AccountIdOf<T> {
		T::PalletId::get().into_sub_account_truncating(community_id)
	}

	pub fn community_exists(community_id: &T::CommunityId) -> bool {
		Self::community(community_id).is_some()
	}

	pub fn has_membership(who: &AccountIdOf<T>, m: MembershipId<T::CommunityId>) -> bool {
		T::Memberships::has_membership(m, who)
	}

	pub fn member_rank(who: &AccountIdOf<T>, m: MembershipId<T::CommunityId>) -> Option<GenericRank> {
		T::Memberships::get_membership(m, who).map(|m| *m.rank())
	}

	pub fn get_memberships(who: &AccountIdOf<T>, community_id: &T::CommunityId) -> Vec<MembershipId<T::CommunityId>> {
		T::Memberships::account_memberships(who)
			.filter(|m| &m.0 == community_id)
			.collect()
	}

	pub fn force_state(community_id: &CommunityIdOf<T>, state: CommunityState) {
		Info::<T>::mutate(community_id, |c| c.as_mut().map(|c| c.state = state));
	}

	/// Stores an initial info about the community
	/// Sets the caller as the community admin, the initial community state
	/// to its default value(awaiting)
	pub(crate) fn do_register_community(admin: &PalletsOriginOf<T>, community_id: &T::CommunityId) -> DispatchResult {
		if Self::community_exists(community_id) {
			return Err(Error::<T>::CommunityAlreadyExists.into());
		}

		CommunityIdFor::<T>::insert(admin, community_id);
		Info::<T>::insert(community_id, CommunityInfo::default());
		frame_system::Pallet::<T>::inc_providers(&Self::community_account(community_id));

		Ok(())
	}

	pub(crate) fn do_set_metadata(
		community_id: &CommunityIdOf<T>,
		name: &Option<ConstSizedField<64>>,
		description: &Option<ConstSizedField<256>>,
		url: &Option<ConstSizedField<256>>,
	) {
		Metadata::<T>::mutate(community_id, |metadata| {
			*metadata = CommunityMetadata {
				name: name.as_ref().unwrap_or(&metadata.name).clone(),
				description: description.as_ref().unwrap_or(&metadata.description).clone(),
				main_url: url.as_ref().unwrap_or(&metadata.main_url).clone(),
			};
		})
	}
}
