use super::*;
use frame_support::traits::Bounded;

/// This structure holds a governance strategy. This defines how to behave
/// when ensuring privileged calls and deciding executing
/// calls
#[derive(TypeInfo, Encode, Decode, MaxEncodedLen, Clone, Eq, PartialEq, Debug)]
#[scale_info(skip_type_params(AccountId, AssetId))]
pub enum CommunityGovernanceStrategy<AccountId, AssetId, VoteWeight> {
	/// The community governance lies in the shoulders of the admin of it.
	///
	/// This is equivalent to `RawOrigin::Member` on collectives-pallet, or
	/// `BodyPart::Voice` on XCM.
	AdminBased(AccountId),
	/// The community governance relies on a member count-based (one member,
	/// one vote) poll.
	///
	/// This is equivalent to `RawOrigin::Members` on collectives-pallet, or
	/// `BodyPart::Members` on XCM.
	MemberCountPoll { min: VoteWeight },
	/// The community governance relies on an asset-weighed (one token,
	/// one vote) poll.
	///
	/// This is equivalent to `RawOrigin::Members` on collectives-pallet, or
	/// `BodyPart::Fraction` on XCM.
	AssetWeighedPoll {
		asset_id: AssetId,
		#[codec(compact)]
		num: VoteWeight,
		#[codec(compact)]
		denum: VoteWeight,
	},
	/// The community governance relies on an ranked-weighed (one member vote,
	/// the number of votes corresponding to the rank of member) poll,
	///
	/// This is equivalent to `RawOrigin::Members` on collectives-pallet, or
	/// `BodyPart::Fraction` on XCM.
	RankedWeighedPoll {
		#[codec(compact)]
		num: VoteWeight,
		#[codec(compact)]
		denum: VoteWeight,
	},
}

/// This structure holds the basic definition of a proposal.
/// It includes the information about the proposer,
/// the hash of the call to be executed if approved,
/// and the information of the
#[derive(TypeInfo, Encode, Decode, MaxEncodedLen, Clone, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(T))]
pub struct CommunityProposal<T: Config> {
	pub(crate) proposer: AccountIdOf<T>,
	pub(crate) call: Bounded<RuntimeCallOf<T>>,
}

/// This structure holds a poll and the methods to increase/decrease
/// votes
#[derive(TypeInfo, Encode, Decode, MaxEncodedLen, Clone)]
#[scale_info(skip_type_params(T))]
pub struct CommunityPoll<T: Config> {
	#[codec(compact)]
	pub(crate) ayes: VoteWeightFor<T>,
	#[codec(compact)]
	pub(crate) nays: VoteWeightFor<T>,
}

impl<T: Config> Default for CommunityPoll<T> {
	fn default() -> Self {
		Self {
			ayes: Default::default(),
			nays: Default::default(),
		}
	}
}

/// This enum defines a vote in a community poll
#[derive(TypeInfo, Encode, Decode, MaxEncodedLen, Clone)]
pub enum CommunityPollVote<T: Config> {
	Aye(VoteWeightFor<T>),
	Nay(VoteWeightFor<T>),
}

/// This enum describes the outcome of a closed poll
pub enum PollOutcome {
	Approved,
	Rejected,
}
