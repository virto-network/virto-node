//! Valiu Liquidity Network - Primitives
//!
//! Common types used to throughout the runtime, might be used by externals to correctly
//! encode/decode messages comming to or from the chain.
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
mod macros;
mod asset;
mod rates;
mod swap;

pub use asset::*;
pub use rates::*;
pub use swap::*;

pub type Share = sp_arithmetic::Permill;
