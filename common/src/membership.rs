use crate::CommunityId;
#[cfg(feature = "runtime")]
use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};

/// Unique identifier of a Virto membership NFT
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "runtime", derive(Decode, Encode, MaxEncodedLen, TypeInfo))]
pub struct MembershipId(CommunityId, MembershipIdPart);

type MembershipIdPart = u32;
type Rank = u8;

/// Detailed information about a membership
#[cfg_attr(feature = "runtime", derive(Decode, Encode, MaxEncodedLen, TypeInfo))]
pub struct MembershipInfo {
	id: MembershipId,
	rank: Rank,
}

impl MembershipInfo {
	pub fn new(id: MembershipId) -> Self {
		Self {
			id,
			rank: Rank::default(),
		}
	}
	pub fn community(&self) -> &CommunityId {
		&self.id.0
	}
}

#[cfg(feature = "runtime")]
mod runtime {
	use super::*;
	use frame_support::traits::membership;

	impl membership::Membership for MembershipInfo {
		type Id = MembershipId;

		fn new(id: Self::Id) -> Self {
			Self::new(id)
		}

		fn id(&self) -> Self::Id {
			self.id
		}
	}

	impl membership::WithRank<Rank> for MembershipInfo {
		fn rank(&self) -> &Rank {
			&self.rank
		}
		fn rank_mut(&mut self) -> &mut Rank {
			&mut self.rank
		}
	}
}
