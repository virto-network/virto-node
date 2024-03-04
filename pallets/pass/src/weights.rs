#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_asset_registry.
pub trait WeightInfo {
	fn call() -> Weight;
}

/// Weights for pallet_asset_registry using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn call() -> Weight {
		0.into()
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn call() -> Weight {
		0.into()
	}
}
