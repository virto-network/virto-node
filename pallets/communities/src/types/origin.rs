use super::*;

/// The origin of the comnunity governance, as well as the origin
/// sent to emit on behalf of the pallet
#[derive(TypeInfo, Encode, Decode, MaxEncodedLen, Clone, Eq, PartialEq, Debug)]
#[scale_info(skip_type_params(T))]
pub struct RawOrigin<T: Config> {
	/// The community id. Used to get the account of the
	/// community for certain origin conversions
	pub community_id: CommunityIdOf<T>,
	///
	pub body_part: BodyPart<VoteWeightFor<T>>,
}

#[derive(TypeInfo, Encode, Decode, MaxEncodedLen, Clone, Eq, PartialEq, Debug)]
#[scale_info(skip_type_params(VotesCount))]
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
