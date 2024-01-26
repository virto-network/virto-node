use crate::{
	types::{
		AccountIdOf, CommunityIdOf, CommunityInfo, CommunityMetadata, CommunityState, ConstSizedField, MembershipIdOf,
		PalletsOriginOf, Tally, VoteWeight,
	},
	CommunityIdFor, Config, Error, Info, Metadata, Pallet,
};
use frame_support::{
	pallet_prelude::*,
	traits::membership::{GenericRank, Inspect, WithRank},
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

	pub fn has_membership(who: &AccountIdOf<T>, m: MembershipIdOf<T>) -> bool {
		T::MemberMgmt::has_membership(m, who)
	}

	pub fn member_rank(who: &AccountIdOf<T>, m: MembershipIdOf<T>) -> Option<GenericRank> {
		T::MemberMgmt::get_membership(m, who).map(|m| m.rank())
	}

	pub fn get_memberships(who: &AccountIdOf<T>, community_id: T::CommunityId) -> Vec<MembershipIdOf<T>> {
		T::MemberMgmt::account_memberships(who)
			.filter(|id| CommunityIdOf::<T>::from(id.clone()) == community_id)
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

impl<T: Config> Tally<T> {
	pub(self) fn add_vote(&mut self, say: bool, weight: VoteWeight) {
		match say {
			true => {
				self.ayes = self.ayes.saturating_add(weight);
				self.bare_ayes = self.bare_ayes.saturating_add(weight);
			}
			false => {
				self.nays = self.nays.saturating_add(weight);
			}
		}
	}

	pub(self) fn remove_vote(&mut self, say: bool, weight: VoteWeight) {
		match say {
			true => {
				self.ayes = self.ayes.saturating_sub(weight);
				self.bare_ayes = self.bare_ayes.saturating_sub(weight);
			}
			false => {
				self.nays = self.nays.saturating_sub(weight);
			}
		}
	}
}
