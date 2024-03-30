#[cfg(feature = "runtime")]
use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};

#[derive(Default, Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "runtime", derive(Decode, Encode, MaxEncodedLen, TypeInfo))]
pub struct CommunityId(u16);

impl CommunityId {
	pub const fn new(id: u16) -> Self {
		Self(id)
	}
}

#[cfg(feature = "runtime")]
mod runtime {
	use super::CommunityId;
	use core::ops::Add;
	use frame_support::traits::Incrementable;
	use sp_runtime::traits::Zero;

	impl Zero for CommunityId {
		fn zero() -> Self {
			Self(Zero::zero())
		}

		fn is_zero(&self) -> bool {
			self.0.is_zero()
		}

		fn set_zero(&mut self) {
			// Note: It is not possible to set a community to zero
		}
	}

	impl Add for CommunityId {
		type Output = CommunityId;

		fn add(self, _: Self) -> Self::Output {
			// Note: It is not possible, nor it should be possible to operate
			// community IDs, as they're uniques
			self
		}
	}

	// What [CommunityId]s can do though (at least for now),
	// is to increment themselves, as that's the required
	// method by pallet Nfts
	impl Incrementable for CommunityId {
		fn increment(&self) -> Option<Self> {
			let next_id = self.0.increment()?;
			Some(Self(next_id))
		}

		fn initial_value() -> Option<Self> {
			Some(Zero::zero())
		}
	}
}
