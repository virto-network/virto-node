//! Virto Network - Primitives
//!
//! Common types used to throughout the runtime, might be used by externals to
//! correctly encode/decode messages comming to or from the chain.
#![cfg_attr(not(feature = "std"), no_std)]

mod asset;

pub use asset::*;
