//! PLACEHOLDER weights for `pallet_migrations`.
//!
//! These are `pallet-migrations` 19.0.0's own `SubstrateWeight` (Parity's reference hardware,
//! `frame-omni-bencher` 32.0.0), not a Kreivo run. Replace this file with the output of
//! `/cmd bench --pallet pallet_migrations` on CCX43 (`F-12`, T-12-04); the bot's commit overwrites it.

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_migrations`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_migrations::WeightInfo for WeightInfo<T> {
	/// Storage: `MultiBlockMigrations::Cursor` (r:1 w:1)
	/// Proof: `MultiBlockMigrations::Cursor` (`max_values`: Some(1), `max_size`: Some(65550), added: 66045, mode: `MaxEncodedLen`)
	/// Storage: UNKNOWN KEY `0x583359fe0e84d953a9dd84e8addb08a5` (r:1 w:0)
	/// Proof: UNKNOWN KEY `0x583359fe0e84d953a9dd84e8addb08a5` (r:1 w:0)
	fn onboard_new_mbms() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `67035`
		// Minimum execution time: 4_411_000 picoseconds.
		Weight::from_parts(4_542_000, 67035)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `MultiBlockMigrations::Cursor` (r:1 w:0)
	/// Proof: `MultiBlockMigrations::Cursor` (`max_values`: Some(1), `max_size`: Some(65550), added: 66045, mode: `MaxEncodedLen`)
	fn progress_mbms_none() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `67035`
		// Minimum execution time: 792_000 picoseconds.
		Weight::from_parts(834_000, 67035)
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	/// Storage: UNKNOWN KEY `0x583359fe0e84d953a9dd84e8addb08a5` (r:1 w:0)
	/// Proof: UNKNOWN KEY `0x583359fe0e84d953a9dd84e8addb08a5` (r:1 w:0)
	/// Storage: `MultiBlockMigrations::Cursor` (r:0 w:1)
	/// Proof: `MultiBlockMigrations::Cursor` (`max_values`: Some(1), `max_size`: Some(65550), added: 66045, mode: `MaxEncodedLen`)
	fn exec_migration_completed() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `3465`
		// Minimum execution time: 3_842_000 picoseconds.
		Weight::from_parts(3_951_000, 3465)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: UNKNOWN KEY `0x583359fe0e84d953a9dd84e8addb08a5` (r:1 w:0)
	/// Proof: UNKNOWN KEY `0x583359fe0e84d953a9dd84e8addb08a5` (r:1 w:0)
	/// Storage: `MultiBlockMigrations::Historic` (r:1 w:0)
	/// Proof: `MultiBlockMigrations::Historic` (`max_values`: None, `max_size`: Some(266), added: 2741, mode: `MaxEncodedLen`)
	fn exec_migration_skipped_historic() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `34`
		//  Estimated: `3731`
		// Minimum execution time: 7_153_000 picoseconds.
		Weight::from_parts(7_407_000, 3731)
			.saturating_add(T::DbWeight::get().reads(2_u64))
	}
	/// Storage: UNKNOWN KEY `0x583359fe0e84d953a9dd84e8addb08a5` (r:1 w:0)
	/// Proof: UNKNOWN KEY `0x583359fe0e84d953a9dd84e8addb08a5` (r:1 w:0)
	/// Storage: `MultiBlockMigrations::Historic` (r:1 w:0)
	/// Proof: `MultiBlockMigrations::Historic` (`max_values`: None, `max_size`: Some(266), added: 2741, mode: `MaxEncodedLen`)
	fn exec_migration_advance() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `3731`
		// Minimum execution time: 6_837_000 picoseconds.
		Weight::from_parts(7_033_000, 3731)
			.saturating_add(T::DbWeight::get().reads(2_u64))
	}
	/// Storage: UNKNOWN KEY `0x583359fe0e84d953a9dd84e8addb08a5` (r:1 w:0)
	/// Proof: UNKNOWN KEY `0x583359fe0e84d953a9dd84e8addb08a5` (r:1 w:0)
	/// Storage: `MultiBlockMigrations::Historic` (r:1 w:1)
	/// Proof: `MultiBlockMigrations::Historic` (`max_values`: None, `max_size`: Some(266), added: 2741, mode: `MaxEncodedLen`)
	fn exec_migration_complete() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `3731`
		// Minimum execution time: 8_167_000 picoseconds.
		Weight::from_parts(8_455_000, 3731)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: UNKNOWN KEY `0x583359fe0e84d953a9dd84e8addb08a5` (r:1 w:0)
	/// Proof: UNKNOWN KEY `0x583359fe0e84d953a9dd84e8addb08a5` (r:1 w:0)
	/// Storage: `MultiBlockMigrations::Historic` (r:1 w:0)
	/// Proof: `MultiBlockMigrations::Historic` (`max_values`: None, `max_size`: Some(266), added: 2741, mode: `MaxEncodedLen`)
	/// Storage: `MultiBlockMigrations::Cursor` (r:0 w:1)
	/// Proof: `MultiBlockMigrations::Cursor` (`max_values`: Some(1), `max_size`: Some(65550), added: 66045, mode: `MaxEncodedLen`)
	fn exec_migration_fail() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `3731`
		// Minimum execution time: 8_868_000 picoseconds.
		Weight::from_parts(9_053_000, 3731)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	fn on_init_loop() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 150_000 picoseconds.
		Weight::from_parts(163_000, 0)
	}
	/// Storage: `MultiBlockMigrations::Cursor` (r:0 w:1)
	/// Proof: `MultiBlockMigrations::Cursor` (`max_values`: Some(1), `max_size`: Some(65550), added: 66045, mode: `MaxEncodedLen`)
	fn force_set_cursor() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 2_142_000 picoseconds.
		Weight::from_parts(2_212_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `MultiBlockMigrations::Cursor` (r:0 w:1)
	/// Proof: `MultiBlockMigrations::Cursor` (`max_values`: Some(1), `max_size`: Some(65550), added: 66045, mode: `MaxEncodedLen`)
	fn force_set_active_cursor() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 2_496_000 picoseconds.
		Weight::from_parts(2_609_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `MultiBlockMigrations::Cursor` (r:1 w:0)
	/// Proof: `MultiBlockMigrations::Cursor` (`max_values`: Some(1), `max_size`: Some(65550), added: 66045, mode: `MaxEncodedLen`)
	/// Storage: UNKNOWN KEY `0x583359fe0e84d953a9dd84e8addb08a5` (r:1 w:0)
	/// Proof: UNKNOWN KEY `0x583359fe0e84d953a9dd84e8addb08a5` (r:1 w:0)
	fn force_onboard_mbms() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `67035`
		// Minimum execution time: 3_010_000 picoseconds.
		Weight::from_parts(3_114_000, 67035)
			.saturating_add(T::DbWeight::get().reads(2_u64))
	}
	/// Storage: `MultiBlockMigrations::Historic` (r:256 w:256)
	/// Proof: `MultiBlockMigrations::Historic` (`max_values`: None, `max_size`: Some(266), added: 2741, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[0, 256]`.
	fn clear_historic(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `960 + n * (271 ±0)`
		//  Estimated: `3834 + n * (2740 ±0)`
		// Minimum execution time: 15_012_000 picoseconds.
		Weight::from_parts(12_864_005, 3834)
			// Standard Error: 3_561
			.saturating_add(Weight::from_parts(1_455_402, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(n.into())))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(n.into())))
			.saturating_add(Weight::from_parts(0, 2740).saturating_mul(n.into()))
	}
	/// Storage: `Skipped::Metadata` (r:0 w:0)
	/// Proof: `Skipped::Metadata` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `n` is `[0, 2048]`.
	fn reset_pallet_migration(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1605 + n * (38 ±0)`
		//  Estimated: `686 + n * (39 ±0)`
		// Minimum execution time: 1_185_000 picoseconds.
		Weight::from_parts(6_858_633, 686)
			// Standard Error: 1_584
			.saturating_add(Weight::from_parts(834_871, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(n.into())))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(n.into())))
			.saturating_add(Weight::from_parts(0, 39).saturating_mul(n.into()))
	}
}
