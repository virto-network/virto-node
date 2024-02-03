use crate::{
	origin::DecisionMethod,
	types::{
		AccountIdOf, CommunityIdOf, CommunityInfo, CommunityMetadata, CommunityState, ConstSizedField, MembershipIdOf,
		PalletsOriginOf, PollIndexOf, Tally, Vote, VoteOf, VoteWeight,
	},
	CommunityDecisionMethod, CommunityIdFor, CommunityVotes, Config, Error, Event, HoldReason, Info, Metadata, Pallet,
};
use frame_support::{
	pallet_prelude::*,
	traits::{
		fungible::MutateFreeze as FunMutateFreeze,
		fungibles::MutateHold as FunsMutateHold,
		membership::{GenericRank, Inspect, WithRank},
		tokens::Precision,
		Polling,
	},
};
use sp_runtime::{traits::AccountIdConversion, TokenError};
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

	pub(crate) fn do_vote(
		who: &AccountIdOf<T>,
		community_id: &CommunityIdOf<T>,
		poll_index: PollIndexOf<T>,
		vote: VoteOf<T>,
	) -> DispatchResult {
		if VoteWeight::from(vote.clone()) == 0 {
			return Err(TokenError::BelowMinimum.into());
		}

		T::Polls::try_access_poll(poll_index, |poll_status| {
			let (tally, class) = poll_status.ensure_ongoing().ok_or(Error::<T>::NotOngoing)?;
			ensure!(community_id == &class, Error::<T>::InvalidTrack);

			let decision_method = CommunityDecisionMethod::<T>::get(community_id);

			let maybe_vote = Self::community_vote_of(who, poll_index);
			if let Some(vote) = maybe_vote {
				Self::do_unlock_for_vote(who, &poll_index, &vote)?;
				tally.remove_vote(vote.clone().into(), vote.into());
			}

			let say = match vote.clone() {
				Vote::AssetBalance(say, asset_id, ..) => {
					ensure!(
						decision_method == DecisionMethod::CommunityAsset(asset_id.clone()),
						Error::<T>::InvalidVoteType
					);

					say
				}
				Vote::NativeBalance(say, ..) => {
					ensure!(
						decision_method == DecisionMethod::NativeToken,
						Error::<T>::InvalidVoteType
					);

					say
				}
				Vote::Standard(say) => {
					ensure!(
						decision_method == DecisionMethod::Membership || decision_method == DecisionMethod::Rank,
						Error::<T>::InvalidVoteType
					);

					say
				}
			};

			Self::do_lock_for_vote(who, &poll_index, &vote)?;
			tally.add_vote(say, vote.clone().into());

			Self::deposit_event(Event::<T>::VoteCasted {
				who: who.clone(),
				poll_index,
				vote,
			});

			Ok(())
		})
	}

	fn do_lock_for_vote(who: &AccountIdOf<T>, poll_index: &PollIndexOf<T>, vote: &VoteOf<T>) -> DispatchResult {
		let reason = HoldReason::VoteCasted(*poll_index).into();
		CommunityVotes::<T>::insert(who.clone(), poll_index, vote.clone());

		match vote {
			Vote::AssetBalance(_, asset_id, amount) => T::Assets::hold(asset_id.clone(), &reason, who, *amount),
			Vote::NativeBalance(_, amount) => {
				T::Balances::set_frozen(&reason, who, *amount, frame_support::traits::tokens::Fortitude::Polite)
			}
			_ => Ok(()),
		}
	}

	pub(crate) fn do_unlock_for_vote(
		who: &AccountIdOf<T>,
		poll_index: &PollIndexOf<T>,
		vote: &VoteOf<T>,
	) -> DispatchResult {
		let reason = HoldReason::VoteCasted(*poll_index).into();
		CommunityVotes::<T>::remove(who, poll_index);

		match vote {
			Vote::AssetBalance(_, asset_id, amount) => {
				T::Assets::release(asset_id.clone(), &reason, who, *amount, Precision::BestEffort).map(|_| ())
			}
			Vote::NativeBalance(..) => T::Balances::thaw(&reason, who),
			_ => Err(Error::<T>::NoLocksInPlace.into()),
		}
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
