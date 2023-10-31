use codec::MaxEncodedLen;
use frame_support::pallet_prelude::{Decode, Encode};
use frame_support::traits::fungible::Inspect;
use frame_support::{sp_runtime::BoundedVec, traits::ConstU32};
use scale_info::{prelude::vec::Vec, TypeInfo};
use sp_runtime::traits::StaticLookup;

use crate::Config;
pub(crate) use frame_system::Config as SystemConfig;

pub use governance::*;
pub use origin::*;
pub use parameters::*;
pub use primitives::*;
pub use registry::*;

mod governance;
mod origin;
mod parameters;
mod primitives;
mod registry;
