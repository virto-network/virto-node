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
		Perbill::from_rational(self.bare_ayes, Self::max_ayes(community_id))
	}

	fn approval(&self, _cid: CommunityIdOf<T>) -> sp_runtime::Perbill {
		Perbill::from_rational(self.ayes, 1.max(self.ayes + self.nays))
	}
}
