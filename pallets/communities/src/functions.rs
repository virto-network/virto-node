use crate::{
	origin::DecisionMethod,
	types::{
		AccountIdOf, CommunityIdOf, CommunityInfo, CommunityState, MembershipIdOf, NativeBalanceOf, PalletsOriginOf,
		PollIndexOf, RuntimeCallFor, Tally, Vote, VoteOf, VoteWeight,
	},
	AssetBalanceOf, AssetIdOf, CommunityDecisionMethod, CommunityIdFor, CommunityVotes, Config, Error, HoldReason,
	Info, Pallet,
};
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

	pub fn register(
		admin: &PalletsOriginOf<T>,
		community_id: &CommunityIdOf<T>,
		maybe_deposit: Option<(NativeBalanceOf<T>, AccountIdOf<T>, AccountIdOf<T>)>,
	) -> DispatchResult {
		if let Some((deposit, depositor, depositee)) = maybe_deposit {
			T::Balances::transfer(
				&depositor,
				&depositee,
				deposit,
				frame_support::traits::tokens::Preservation::Preserve,
			)?;
		}

		Self::do_register_community(admin, community_id)
	}

	/// Stores an initial info about the community
	/// Sets the caller as the community admin, the initial community state
	/// to its default value(awaiting)
	pub(crate) fn do_register_community(admin: &PalletsOriginOf<T>, community_id: &T::CommunityId) -> DispatchResult {
		ensure!(
			!Self::community_exists(community_id),
			Error::<T>::CommunityAlreadyExists
		);

		CommunityIdFor::<T>::insert(admin, community_id);
		Info::<T>::insert(community_id, CommunityInfo::default());
		frame_system::Pallet::<T>::inc_providers(&Self::community_account(community_id));

		Self::deposit_event(crate::Event::CommunityCreated {
			id: *community_id,
			origin: admin.clone(),
		});

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
			Self::update_locks(who)
		})
	}

	pub(crate) fn try_remove_vote(
		who: &AccountIdOf<T>,
		membership_id: MembershipIdOf<T>,
		poll_index: PollIndexOf<T>,
	) -> DispatchResult {
		let community_id = T::MemberMgmt::check_membership(who, &membership_id).ok_or(Error::<T>::NotAMember)?;

		T::Polls::try_access_poll(poll_index, |poll_status| {
			let Some((tally, class)) = poll_status.ensure_ongoing() else {
				fail!(Error::<T>::NotOngoing);
			};
			ensure!(community_id == class, Error::<T>::InvalidTrack);

			let (vote, _) = CommunityVotes::<T>::get(poll_index, membership_id).ok_or(Error::<T>::NoVoteCasted)?;
			let vote_multiplier = match CommunityDecisionMethod::<T>::get(community_id) {
				DecisionMethod::Rank => T::MemberMgmt::rank_of(&community_id, &membership_id)
					.unwrap_or_default()
					.into(),
				_ => 1,
			};

			let vote_weight = VoteWeight::from(&vote);
			tally.remove_vote(vote.say(), vote_multiplier * vote_weight, vote_weight);

			CommunityVotes::<T>::remove(poll_index, membership_id);
			Self::update_locks(who)
		})
	}

	pub(crate) fn do_unlock(
		who: &AccountIdOf<T>,
		membership_id: MembershipIdOf<T>,
		poll_index: PollIndexOf<T>,
	) -> DispatchResult {
		T::MemberMgmt::check_membership(who, &membership_id).ok_or(Error::<T>::NotAMember)?;
		CommunityVotes::<T>::remove(poll_index, membership_id);
		Self::update_locks(who)
	}

	pub(crate) fn update_locks(who: &AccountIdOf<T>) -> DispatchResult {
		use sp_runtime::traits::Zero;
		let reason = HoldReason::VoteCasted.into();

		let mut assets_locked_amount: Vec<(AssetIdOf<T>, AssetBalanceOf<T>)> = vec![];
		let mut native_locked_amount: NativeBalanceOf<T> = Zero::zero();

		for (locked_vote, _) in CommunityVotes::<T>::iter_values().filter(|(_, voter)| voter == who) {
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
