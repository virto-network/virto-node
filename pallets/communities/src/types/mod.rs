use core::ops::Deref;

use frame_support::pallet_prelude::{Decode, Encode};
use frame_support::traits::fungible::Inspect;
use frame_support::{sp_runtime::BoundedVec, traits::ConstU32};
use parity_scale_codec::MaxEncodedLen;
use scale_info::{prelude::vec::Vec, TypeInfo};
use sp_runtime::traits::StaticLookup;

use crate::Config;
pub(crate) use frame_system::Config as SystemConfig;

pub use governance::*;
pub use origin::*;
pub use parameters::*;
pub use registry::*;

mod governance;
mod origin;
mod parameters;
mod registry;

pub type SizedField<S> = BoundedVec<u8, S>;
pub type ConstSizedField<const S: u32> = SizedField<ConstU32<S>>;

/// A general purpose rank in the range 0-100
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Rank(u8);

impl Rank {
	pub const MAX: Self = Rank(100);
	pub const MIN: Self = Rank(0);

	#[inline]
	pub fn promote(&mut self) {
		*self = self.0.saturating_add(1).min(Self::MAX.0).into()
	}

	#[inline]
	pub fn demote(&mut self) {
		*self = self.0.saturating_sub(1).max(Self::MIN.0).into()
	}
}

impl From<u8> for Rank {
	fn from(rank: u8) -> Self {
		Rank(rank)
	}
}
impl Deref for Rank {
	type Target = u8;
	fn deref(&self) -> &Self::Target {
		&(self.0)
	}
}
