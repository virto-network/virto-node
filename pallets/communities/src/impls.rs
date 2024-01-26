use frame_support::traits::VoteTally;
use sp_runtime::Perbill;

use crate::{
	types::{Tally, VoteWeight},
	Config,
};

impl<T: Config> VoteTally<VoteWeight, T::CommunityId> for Tally<T> {
	fn new(_cid: T::CommunityId) -> Self {
		Self::default()
	}

	fn ayes(&self, _cid: T::CommunityId) -> VoteWeight {
		self.ayes
	}

	fn support(&self, community_id: T::CommunityId) -> sp_runtime::Perbill {
		Perbill::from_rational(self.bare_ayes, Self::max_ayes(community_id))
	}

	fn approval(&self, community_id: T::CommunityId) -> sp_runtime::Perbill {
		sp_runtime::Perbill::from_rational(self.ayes, Self::max_ayes(community_id))
	}
}
