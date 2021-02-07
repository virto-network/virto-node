//! Valiu Liquidity Network - Commons
//!
//! Common structures shared between the different Valiu Liquidity Network projects.

#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
mod macros;

pub mod runtime;

mod account_rate;
mod asset;
mod collateral;
mod destination;
mod offer_rate;
mod pair;
mod pair_price;

pub use account_rate::*;
pub use asset::*;
pub use collateral::*;
pub use destination::*;
pub use offer_rate::*;
pub use pair::*;
pub use pair_price::*;
