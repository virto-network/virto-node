#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![cfg_attr(feature = "nightly", feature(ascii_char))]

#[cfg(feature = "alloc")]
extern crate alloc;
extern crate sp_io;

#[cfg(feature = "runtime")]
use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};

mod membership;
mod payment_id;
pub use membership::{MembershipId, MembershipInfo};
pub use payment_id::PaymentId;

#[cfg_attr(feature = "runtime", derive(Decode, Encode, MaxEncodedLen, TypeInfo))]
#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct CommunityId(u16);
