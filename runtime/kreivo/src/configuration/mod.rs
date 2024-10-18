//! Configure FRAME pallets to include in runtime.

use super::*;

pub mod _0_system_support;
pub mod _10_monetary_stuff;

pub use _10_monetary_stuff::{KreivoAssetsCall, KreivoAssetsInstance, MetadataDepositBase, MetadataDepositPerByte};
