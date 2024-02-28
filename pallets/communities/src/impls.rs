use frame_support::traits::VoteTally;
use sp_runtime::Perbill;

use crate::{
	types::{CommunityIdOf, Tally, VoteWeight},
	Config,
};

#[cfg(feature = "runtime-benchmarks")]
use crate::{CommunityIdFor, Pallet};
#[cfg(feature = "runtime-benchmarks")]
use frame_support::traits::membership::Inspect;

impl<T: Config> VoteTally<VoteWeight, CommunityIdOf<T>> for Tally<T> {
	fn new(_cid: CommunityIdOf<T>) -> Self {
		Self::default()
	}

	fn ayes(&self, _cid: CommunityIdOf<T>) -> VoteWeight {
		self.ayes
	}

	fn support(&self, community_id: CommunityIdOf<T>) -> sp_runtime::Perbill {
		Perbill::from_rational(self.bare_ayes, Self::max_support(community_id))
	}

	fn approval(&self, _cid: CommunityIdOf<T>) -> sp_runtime::Perbill {
		Perbill::from_rational(self.ayes, 1.max(self.ayes + self.nays))
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn unanimity(community_id: CommunityIdOf<T>) -> Self {
		Self {
			ayes: Self::max_support(community_id),
			bare_ayes: Self::max_support(community_id),
			nays: 0,
			..Default::default()
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn rejection(community_id: CommunityIdOf<T>) -> Self {
		Self {
			ayes: 0,
			bare_ayes: 0,
			nays: Self::max_support(community_id),
			..Default::default()
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn from_requirements(support: Perbill, approval: Perbill, community_id: CommunityIdOf<T>) -> Self {
		let approval_weight = approval * Self::max_support(community_id);
		let rejection_weight = (Perbill::from_percent(100) - approval) * Self::max_support(community_id);
		let support_weight = support * Self::max_support(community_id);

		Self {
			ayes: approval_weight,
			nays: rejection_weight,
			bare_ayes: support_weight,
			..Default::default()
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn setup(community_id: CommunityIdOf<T>, granularity: Perbill) {
		let community_account_id = Pallet::<T>::community_account(&community_id);
		let community_origin = CommunityIdFor::<T>::iter_keys()
			.find(|key| CommunityIdFor::<T>::get(key) == Some(community_id))
			.expect("find a community origin by its id");
		let origin = community_origin.into();
		let voters = granularity.saturating_reciprocal_mul(1u32);

		// Worst case scenarios can be best assessed via Rank decision method
		for i in 0..voters {
			let account_id: T::AccountId = frame_benchmarking::account("ranked_collective_benchmarking", i, 0);
			let who = T::Lookup::unlookup(account_id);
			let membership_id = T::MemberMgmt::account_memberships(&community_account_id)
				.next()
				.expect("has enough memberships");
			Pallet::<T>::add_member(origin.clone(), who.clone()).expect("can add member to community");
			Pallet::<T>::promote_member(origin.clone(), who, membership_id).expect("can add member to community");
		}
	}
}
