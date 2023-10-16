use super::*;

/// The origin of the comnunity governance, as well as the origin
/// sent to emit on behalf of the pallet
#[derive(TypeInfo, Encode, Decode, MaxEncodedLen, Clone, Eq, PartialEq, Debug)]
pub struct RawOrigin<CommunityId, VoteWeight>
where
	CommunityId: TypeInfo + MaxEncodedLen,
	VoteWeight: TypeInfo + MaxEncodedLen,
{
	/// The community id. Used to get the account of the
	/// community for certain origin conversions
	pub community_id: CommunityId,
	///
	pub body_part: BodyPart<VoteWeight>,
}

#[derive(TypeInfo, Encode, Decode, MaxEncodedLen, Clone, Eq, PartialEq, Debug)]
pub enum BodyPart<VotesCount: TypeInfo + MaxEncodedLen> {
	Voice,
	Members {
		#[codec(compact)]
		min: VotesCount,
	},
	Fraction {
		#[codec(compact)]
		num: VotesCount,
		#[codec(compact)]
		denum: VotesCount,
	},
}
