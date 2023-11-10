use super::*;

use frame_support::traits::VoteTally;

impl<A, B> From<Vote<A, B>> for VoteWeight {
	fn from(_value: Vote<A, B>) -> Self {
		todo!()
	}
}

impl<T: Config> VoteTally<VoteWeight, CommunityIdOf<T>> for Tally<T> {
	fn new(_: CommunityIdOf<T>) -> Self {
		todo!()
	}

	fn ayes(&self, _cid: CommunityIdOf<T>) -> VoteWeight {
		todo!()
	}

	fn support(&self, _cid: CommunityIdOf<T>) -> sp_runtime::Perbill {
		todo!()
	}

	fn approval(&self, _cid: CommunityIdOf<T>) -> sp_runtime::Perbill {
		todo!()
	}
}
