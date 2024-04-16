#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_communities.
pub trait WeightInfo {
	fn register() -> Weight;
	fn configure_track() -> Weight;
}

/// Weights for pallet_communities using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
  fn register() -> Weight {
    Weight::from_parts(18_000_000, 4087)
      .saturating_add(T::DbWeight::get().reads(2_u64))
      .saturating_add(T::DbWeight::get().writes(2_u64))
  }
  
  fn configure_track() -> Weight {
    Weight::from_parts(18_000_000, 4087)
      .saturating_add(T::DbWeight::get().reads(2_u64))
      .saturating_add(T::DbWeight::get().writes(2_u64))
  }
}

impl WeightInfo for () {
  fn register() -> Weight {
    Weight::from_parts(18_000_000, 4087)
      .saturating_add(RocksDbWeight::get().reads(2_u64))
      .saturating_add(RocksDbWeight::get().writes(2_u64))
  }

  fn configure_track() -> Weight {
    Weight::from_parts(18_000_000, 4087)
      .saturating_add(RocksDbWeight::get().reads(2_u64))
      .saturating_add(RocksDbWeight::get().writes(2_u64))
  }
}