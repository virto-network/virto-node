use crate::{
	origin::DecisionMethod,
	types::{
		AccountIdOf, CommunityIdOf, CommunityInfo, CommunityState, ConstSizedField, MembershipIdOf, PalletsOriginOf,
		PollIndexOf, Tally, Vote, VoteOf, VoteWeight,
	},
	CommunityDecisionMethod, CommunityIdFor, CommunityVotes, Config, Error, HoldReason, Info, Metadata, Pallet,
};
use fc_traits_memberships::{GenericRank, Inspect, Rank};
use frame_support::{
	fail,
	pallet_prelude::*,
	traits::{
		fungible::MutateFreeze as FunMutateFreeze, fungibles::MutateHold as FunsMutateHold, tokens::Precision, Polling,
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

	pub fn is_member(community_id: &T::CommunityId, who: &AccountIdOf<T>) -> bool {
		T::MemberMgmt::is_member_of(community_id, who)
	}

	pub fn member_rank(community_id: &T::CommunityId, m: &MembershipIdOf<T>) -> GenericRank {
		T::MemberMgmt::rank_of(community_id, m).unwrap_or_default()
	}

	pub fn get_memberships(community_id: T::CommunityId, who: &AccountIdOf<T>) -> Vec<MembershipIdOf<T>> {
		T::MemberMgmt::user_memberships(who, Some(community_id))
			.map(|(_, m)| m)
			.collect::<Vec<_>>()
	}

	pub fn force_state(community_id: &CommunityIdOf<T>, state: CommunityState) {
		Info::<T>::mutate(community_id, |c| c.as_mut().map(|c| c.state = state));
	}

	/// Stores an initial info about the community
	/// Sets the caller as the community admin, the initial community state
	/// to its default value(awaiting)
	pub(crate) fn do_register_community(admin: &PalletsOriginOf<T>, community_id: &T::CommunityId) -> DispatchResult {
		if Self::community_exists(community_id) {
			fail!(Error::<T>::CommunityAlreadyExists);
		}

		CommunityIdFor::<T>::insert(admin, community_id);
		Info::<T>::insert(community_id, CommunityInfo::default());
		frame_system::Pallet::<T>::inc_providers(&Self::community_account(community_id));

		Ok(())
	}

	pub(crate) fn do_set_metadata(
		community_id: &CommunityIdOf<T>,
		name: &Option<ConstSizedField<64>>,
		description: Option<ConstSizedField<256>>,
		url: Option<ConstSizedField<256>>,
	) {
		Metadata::<T>::mutate(community_id, |metadata| {
			if let Some(name) = name {
				metadata.name = name.clone();
			}
			if let Some(desc) = description {
				metadata.description = desc;
			}
			if let Some(url) = url {
				metadata.main_url = url;
			}
		})
	}

	pub(crate) fn do_vote(
		who: &AccountIdOf<T>,
		membership_id: MembershipIdOf<T>,
		poll_index: PollIndexOf<T>,
		vote: &VoteOf<T>,
	) -> DispatchResult {
		ensure!(VoteWeight::from(vote).gt(&0), TokenError::BelowMinimum);
		let Some(community_id) = T::MemberMgmt::check_membership(who, &membership_id) else {
			fail!(Error::<T>::NotAMember);
		};

		T::Polls::try_access_poll(poll_index, |poll_status| {
			let (tally, class) = poll_status.ensure_ongoing().ok_or(Error::<T>::NotOngoing)?;
			ensure!(community_id == class, Error::<T>::InvalidTrack);

			let decision_method = CommunityDecisionMethod::<T>::get(community_id);

			let vote_multiplier = match CommunityDecisionMethod::<T>::get(community_id) {
				DecisionMethod::Rank => T::MemberMgmt::rank_of(&community_id, &membership_id)
					.unwrap_or_default()
					.into(),
				_ => 1,
			};
			if let Some(vote) = Self::community_vote_of(who, poll_index) {
				Self::do_unlock_for_vote(who, &poll_index, &vote)?;
				let vote_weight = VoteWeight::from(&vote);
				tally.remove_vote(vote.say(), vote_multiplier * vote_weight, vote_weight);
			}

			let say = *match (vote, decision_method) {
				(Vote::AssetBalance(say, asset, ..), DecisionMethod::CommunityAsset(a)) if *asset == a => say,
				(Vote::NativeBalance(say, ..), DecisionMethod::NativeToken)
				| (Vote::Standard(say), DecisionMethod::Membership | DecisionMethod::Rank) => say,
				_ => fail!(Error::<T>::InvalidVoteType),
			};

			let vote_weight = VoteWeight::from(vote);
			tally.add_vote(say, vote_multiplier * vote_weight, vote_weight);

			Self::do_lock_for_vote(who, &poll_index, vote)
		})
	}

	pub(crate) fn do_remove_vote(
		who: &AccountIdOf<T>,
		membership_id: MembershipIdOf<T>,
		poll_index: PollIndexOf<T>,
	) -> DispatchResult {
		let Some(community_id) = T::MemberMgmt::check_membership(who, &membership_id) else {
			fail!(Error::<T>::NotAMember);
		};

		T::Polls::try_access_poll(poll_index, |poll_status| {
			let Some((tally, class)) = poll_status.ensure_ongoing() else {
				fail!(Error::<T>::NotOngoing);
			};
			ensure!(community_id == class, Error::<T>::InvalidTrack);
			let vote = Self::community_vote_of(who, poll_index).ok_or(Error::<T>::NoVoteCasted)?;

			let vote_multiplier = match CommunityDecisionMethod::<T>::get(community_id) {
				DecisionMethod::Rank => T::MemberMgmt::rank_of(&community_id, &membership_id)
					.unwrap_or_default()
					.into(),
				_ => 1,
			};
			let vote_weight = VoteWeight::from(&vote);
			tally.remove_vote(vote.say(), vote_multiplier * vote_weight, vote_weight);

			let reason = HoldReason::VoteCasted(poll_index).into();
			CommunityVotes::<T>::remove(who, poll_index);

			match vote {
				Vote::AssetBalance(_, asset_id, amount) => {
					T::Assets::release(asset_id.clone(), &reason, who, amount, Precision::BestEffort).map(|_| ())
				}
				Vote::NativeBalance(..) => T::Balances::thaw(&reason, who),
				_ => Ok(()),
			}
		})
	}

	fn do_lock_for_vote(who: &AccountIdOf<T>, poll_index: &PollIndexOf<T>, vote: &VoteOf<T>) -> DispatchResult {
		let reason = HoldReason::VoteCasted(*poll_index).into();
		CommunityVotes::<T>::insert(who, poll_index, vote.clone());

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
	pub(self) fn add_vote(&mut self, say_aye: bool, multiplied_weight: VoteWeight, weight: VoteWeight) {
		if say_aye {
			self.ayes = self.ayes.saturating_add(multiplied_weight);
			self.bare_ayes = self.bare_ayes.saturating_add(weight);
		} else {
			self.nays = self.nays.saturating_add(multiplied_weight);
		}
	}

	pub(self) fn remove_vote(&mut self, say_aye: bool, multiplied_weight: VoteWeight, weight: VoteWeight) {
		if say_aye {
			self.ayes = self.ayes.saturating_sub(multiplied_weight);
			self.bare_ayes = self.bare_ayes.saturating_sub(weight);
		} else {
			self.nays = self.nays.saturating_sub(multiplied_weight);
		}
	}
}
