use super::{origin::DecisionMethod, *};
use fc_traits_memberships::{GenericRank, Inspect, Rank};
use frame_support::{
	dispatch::PostDispatchInfo,
	fail,
	pallet_prelude::*,
	traits::{
		fungible::{Mutate, MutateFreeze},
		fungibles::{InspectHold, MutateHold},
		tokens::Precision,
		Polling,
	},
};
use sp_runtime::{
	traits::{AccountIdConversion, Dispatchable},
	DispatchResultWithInfo, TokenError,
};
use sp_std::{vec, vec::Vec};

impl<T: Config> Pallet<T> {
	#[inline]
	pub fn community_account(community_id: &T::CommunityId) -> AccountIdOf<T> {
		T::PalletId::get().into_sub_account_truncating(community_id)
	}

	pub fn community_exists(community_id: &T::CommunityId) -> bool {
		Info::<T>::contains_key(community_id)
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
	pub fn register(
		admin: &PalletsOriginOf<T>,
		community_id: &CommunityIdOf<T>,
		maybe_deposit: Option<(NativeBalanceOf<T>, AccountIdOf<T>, AccountIdOf<T>)>,
	) -> DispatchResult {
		ensure!(
			!Self::community_exists(community_id),
			Error::<T>::CommunityAlreadyExists
		);

		if let Some((deposit, depositor, depositee)) = maybe_deposit {
			T::Balances::transfer(
				&depositor,
				&depositee,
				deposit,
				frame_support::traits::tokens::Preservation::Preserve,
			)?;
		}

		CommunityIdFor::<T>::insert(admin, community_id);
		Info::<T>::insert(community_id, CommunityInfo::default());
		frame_system::Pallet::<T>::inc_providers(&Self::community_account(community_id));

		Ok(())
	}

	pub(crate) fn try_vote(
		who: &AccountIdOf<T>,
		membership_id: MembershipIdOf<T>,
		poll_index: PollIndexOf<T>,
		vote: &VoteOf<T>,
	) -> DispatchResult {
		ensure!(VoteWeight::from(vote).gt(&0), TokenError::BelowMinimum);
		let community_id = T::MemberMgmt::check_membership(who, &membership_id).ok_or(Error::<T>::NotAMember)?;

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

			let say = *match (vote, decision_method) {
				(Vote::AssetBalance(say, asset, ..), DecisionMethod::CommunityAsset(a)) if *asset == a => say,
				(Vote::NativeBalance(say, ..), DecisionMethod::NativeToken)
				| (Vote::Standard(say), DecisionMethod::Membership | DecisionMethod::Rank) => say,
				_ => fail!(Error::<T>::InvalidVoteType),
			};

			let vote_weight = VoteWeight::from(vote);
			tally.add_vote(say, vote_multiplier * vote_weight, vote_weight);

			CommunityVotes::<T>::insert(poll_index, membership_id, (vote, who));
			Self::update_locks(who, poll_index, vote, LockUpdateType::Add)
		})
	}

	pub(crate) fn try_remove_vote(
		who: &AccountIdOf<T>,
		membership_id: MembershipIdOf<T>,
		poll_index: PollIndexOf<T>,
	) -> DispatchResult {
		let community_id = T::MemberMgmt::check_membership(who, &membership_id).ok_or(Error::<T>::NotAMember)?;

		T::Polls::try_access_poll(poll_index, |poll_status| {
			let (tally, class) = poll_status.ensure_ongoing().ok_or(Error::<T>::NotOngoing)?;
			ensure!(community_id == class, Error::<T>::InvalidTrack);

			let (vote, voter) = CommunityVotes::<T>::get(poll_index, membership_id).ok_or(Error::<T>::NoVoteCasted)?;
			let vote_multiplier = match CommunityDecisionMethod::<T>::get(community_id) {
				DecisionMethod::Rank => T::MemberMgmt::rank_of(&community_id, &membership_id)
					.unwrap_or_default()
					.into(),
				_ => 1,
			};

			let vote_weight = VoteWeight::from(&vote);
			tally.remove_vote(vote.say(), vote_multiplier * vote_weight, vote_weight);

			CommunityVotes::<T>::remove(poll_index, membership_id);
			Self::update_locks(&voter, poll_index, &vote, LockUpdateType::Remove)
		})
	}

	pub(crate) fn update_locks(
		who: &AccountIdOf<T>,
		poll_index: PollIndexOf<T>,
		vote: &VoteOf<T>,
		update_type: LockUpdateType,
	) -> DispatchResult {
		use sp_runtime::traits::Zero;
		let reason = HoldReason::VoteCasted.into();

		// 1. Define locks
		let mut assets_locked_amount: Vec<(AssetIdOf<T>, AssetBalanceOf<T>)> = vec![];
		let mut native_locked_amount: NativeBalanceOf<T> = Zero::zero();

		// 1a. Define possibly asset to void
		if let Vote::AssetBalance(_, asset_id, _) = vote {
			assets_locked_amount.push((asset_id.clone(), Zero::zero()));
		}

		match update_type {
			LockUpdateType::Add => CommunityVoteLocks::<T>::insert(who, poll_index, vote),
			LockUpdateType::Remove => CommunityVoteLocks::<T>::remove(who, poll_index),
		}

		for locked_vote in CommunityVoteLocks::<T>::iter_prefix_values(who) {
			match locked_vote {
				Vote::AssetBalance(_, asset_id, amount) => {
					if let Some((_, locked_amount)) =
						assets_locked_amount.iter_mut().find(|(asset, _)| asset == &asset_id)
					{
						*locked_amount = (*locked_amount).max(amount);
					} else {
						assets_locked_amount.push((asset_id.clone(), amount));
					}
				}
				Vote::NativeBalance(_, amount) => native_locked_amount = native_locked_amount.max(amount),
				_ => (),
			}
		}

		// 3. Apply locks
		for (asset, amount) in assets_locked_amount.iter() {
			let held_balance = T::Assets::balance_on_hold(asset.clone(), &reason, who);
			if held_balance.gt(&Zero::zero()) {
				T::Assets::release(asset.clone(), &reason, who, held_balance, Precision::Exact)?;
			}
			T::Assets::hold(asset.clone(), &reason, who, *amount)?;
		}

		T::Balances::set_frozen(
			&reason,
			who,
			native_locked_amount,
			frame_support::traits::tokens::Fortitude::Polite,
		)?;

		Ok(())
	}

	pub(crate) fn do_dispatch_as_community_account(
		community_id: &CommunityIdOf<T>,
		call: RuntimeCallFor<T>,
	) -> DispatchResultWithInfo<PostDispatchInfo> {
		let community_account = Self::community_account(community_id);
		let signer = frame_system::RawOrigin::Signed(community_account);

		let post = call.dispatch(signer.into()).map_err(|e| e.error)?;
		Ok(post)
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
