#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![cfg_attr(feature = "nightly", feature(ascii_char))]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::ops::Add;

#[cfg(feature = "runtime")]
use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};

mod membership;
mod multilocation_asset_id;
mod payment_id;
pub use membership::{MembershipId, MembershipInfo};
pub use multilocation_asset_id::{FungibleAssetLocation, NetworkId};
pub use payment_id::PaymentId;

#[cfg(feature = "runtime")]
pub use multilocation_asset_id::runtime::AsFungibleAssetLocation;

#[cfg_attr(feature = "runtime", derive(Decode, Encode, MaxEncodedLen, TypeInfo))]
#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct CommunityId(u16);

impl CommunityId {
	pub const fn new(id: u16) -> Self {
		Self(id)
	}
}

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
		self.0.increment().map(|i| Self(i))
	}

	fn initial_value() -> Option<Self> {
		Some(Zero::zero())
	}
}
