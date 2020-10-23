#![cfg_attr(not(feature = "std"), no_std)]

mod asset;
mod collateral;

pub use asset::*;
pub use collateral::*;
