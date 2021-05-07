//! Valiu Liquidity Network - Primitives
//!
//! Common types used to throughout the runtime, might be used by externals to correctly
//! encode/decode messages comming to or from the chain.
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
mod macros;
mod asset;
mod escrow;
mod rates;

pub use asset::*;
pub use escrow::*;
pub use rates::*;

pub type Share = sp_arithmetic::Permill;
