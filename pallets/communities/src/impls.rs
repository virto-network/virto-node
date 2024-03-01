use frame_support::traits::VoteTally;
use sp_runtime::Perbill;

use crate::{
	types::{CommunityIdOf, Tally, VoteWeight},
	Config,
};

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
	fn setup(_community_id: CommunityIdOf<T>, _granularity: Perbill) {}
}
