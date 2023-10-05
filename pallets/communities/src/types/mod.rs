use codec::MaxEncodedLen;
use frame_support::pallet_prelude::{Decode, Encode};
use frame_support::traits::{fungible::Inspect, fungibles::Inspect as InspectFuns};
use frame_support::{sp_runtime::BoundedVec, traits::ConstU32};
use scale_info::{prelude::vec::Vec, TypeInfo};
use sp_runtime::traits::StaticLookup;

use crate::Config;
use frame_system::Config as SystemConfig;

pub use parameters::*;
pub use primitives::*;
pub use registry::*;

mod parameters;
mod primitives;
mod registry;