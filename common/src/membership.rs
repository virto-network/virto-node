use crate::CommunityId;
#[cfg(feature = "runtime")]
use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};

type MembershipIdPart = u32;
type Rank = u8;

/// Unique identifier of a Virto membership NFT
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "runtime", derive(Decode, Encode, MaxEncodedLen, TypeInfo))]
pub struct MembershipId(pub CommunityId, pub MembershipIdPart);

impl From<MembershipId> for CommunityId {
	fn from(id: MembershipId) -> Self {
		id.0
	}
}

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
	pub fn rank(&self) -> Rank {
		self.rank
	}
}

#[cfg(feature = "runtime")]
mod runtime {
	use super::*;
	use frame_support::traits::membership::{self, GenericRank};

	impl membership::Membership for MembershipInfo {
		type Id = MembershipId;

		fn new(id: Self::Id) -> Self {
			Self::new(id)
		}

		fn id(&self) -> Self::Id {
			self.id
		}
	}

	impl membership::WithRank<GenericRank> for MembershipInfo {
		fn rank(&self) -> GenericRank {
			self.rank.into()
		}
		fn set_rank(&mut self, rank: impl Into<GenericRank>) {
			self.rank = u8::from(rank.into());
		}
	}
}
