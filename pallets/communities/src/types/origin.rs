use super::*;

/// The origin of the comnunity governance, as well as the origin
/// sent to emit on behalf of the pallet
#[derive(TypeInfo, Encode, Decode, MaxEncodedLen, Clone, Eq, PartialEq, Debug)]
pub struct RawOrigin<CommunityId> {
	/// The community id. Used to get the account of the
	/// community for certain origin conversions
	pub community_id: CommunityId,
	///
	pub body_part: BodyPart,
}

#[derive(TypeInfo, Encode, Decode, MaxEncodedLen, Clone, Eq, PartialEq, Debug)]
pub enum BodyPart {
	Voice,
	Members {
		#[codec(compact)]
		min: VoteWeight,
	},
	Fraction {
		#[codec(compact)]
		num: u16,
		#[codec(compact)]
		denum: u16,
	},
}
