
//! Autogenerated weights for `pallet_communities`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-03-30, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `virto-builder`, CPU: `Intel(R) Xeon(R) Silver 4216 CPU @ 2.10GHz`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("kreivo-local")`, DB CACHE: 1024

// Executed Command:
// ./target/release/virto-node
// benchmark
// pallet
// --chain
// kreivo-local
// --pallet
// pallet_communities
// --extrinsic
// *
// --steps
// 50
// --repeat
// 20
// --output
// runtime/kreivo/src/weights/pallet_communities.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_communities`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_communities::WeightInfo for WeightInfo<T> {
	/// Storage: `Communities::Info` (r:1 w:1)
	/// Proof: `Communities::Info` (`max_values`: None, `max_size`: Some(19), added: 2494, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Communities::CommunityIdFor` (r:0 w:1)
	/// Proof: `Communities::CommunityIdFor` (`max_values`: None, `max_size`: Some(622), added: 3097, mode: `MaxEncodedLen`)
	fn create() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `56`
		//  Estimated: `3593`
		// Minimum execution time: 57_149_000 picoseconds.
		Weight::from_parts(58_958_000, 0)
			.saturating_add(Weight::from_parts(0, 3593))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `Communities::Metadata` (r:1 w:1)
	/// Proof: `Communities::Metadata` (`max_values`: None, `max_size`: Some(600), added: 3075, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[1, 64]`.
	/// The range of component `d` is `[1, 256]`.
	/// The range of component `u` is `[1, 256]`.
	fn set_metadata(n: u32, d: u32, u: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `75`
		//  Estimated: `4065`
		// Minimum execution time: 22_806_000 picoseconds.
		Weight::from_parts(29_229_606, 0)
			.saturating_add(Weight::from_parts(0, 4065))
			// Standard Error: 5_937
			.saturating_add(Weight::from_parts(18_897, 0).saturating_mul(n.into()))
			// Standard Error: 1_475
			.saturating_add(Weight::from_parts(7_074, 0).saturating_mul(d.into()))
			// Standard Error: 1_475
			.saturating_add(Weight::from_parts(11_421, 0).saturating_mul(u.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Communities::CommunityDecisionMethod` (r:0 w:1)
	/// Proof: `Communities::CommunityDecisionMethod` (`max_values`: None, `max_size`: Some(36), added: 2511, mode: `MaxEncodedLen`)
	fn set_decision_method() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 22_228_000 picoseconds.
		Weight::from_parts(24_617_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Communities::Info` (r:1 w:0)
	/// Proof: `Communities::Info` (`max_values`: None, `max_size`: Some(19), added: 2494, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Account` (r:1 w:2)
	/// Proof: `CommunityMemberships::Account` (`max_values`: None, `max_size`: Some(88), added: 2563, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Attribute` (r:3 w:1)
	/// Proof: `CommunityMemberships::Attribute` (`max_values`: None, `max_size`: Some(479), added: 2954, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::ItemConfigOf` (r:2 w:2)
	/// Proof: `CommunityMemberships::ItemConfigOf` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Collection` (r:2 w:2)
	/// Proof: `CommunityMemberships::Collection` (`max_values`: None, `max_size`: Some(82), added: 2557, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Item` (r:2 w:2)
	/// Proof: `CommunityMemberships::Item` (`max_values`: None, `max_size`: Some(861), added: 3336, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::ItemMetadataOf` (r:1 w:0)
	/// Proof: `CommunityMemberships::ItemMetadataOf` (`max_values`: None, `max_size`: Some(347), added: 2822, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::CollectionConfigOf` (r:1 w:0)
	/// Proof: `CommunityMemberships::CollectionConfigOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `Communities::CommunityRanksSum` (r:1 w:1)
	/// Proof: `Communities::CommunityRanksSum` (`max_values`: None, `max_size`: Some(22), added: 2497, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::ItemPriceOf` (r:0 w:1)
	/// Proof: `CommunityMemberships::ItemPriceOf` (`max_values`: None, `max_size`: Some(89), added: 2564, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::ItemAttributesApprovalsOf` (r:0 w:1)
	/// Proof: `CommunityMemberships::ItemAttributesApprovalsOf` (`max_values`: None, `max_size`: Some(1001), added: 3476, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::PendingSwapOf` (r:0 w:1)
	/// Proof: `CommunityMemberships::PendingSwapOf` (`max_values`: None, `max_size`: Some(71), added: 2546, mode: `MaxEncodedLen`)
	fn add_member() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1129`
		//  Estimated: `9852`
		// Minimum execution time: 402_307_000 picoseconds.
		Weight::from_parts(451_480_000, 0)
			.saturating_add(Weight::from_parts(0, 9852))
			.saturating_add(T::DbWeight::get().reads(16))
			.saturating_add(T::DbWeight::get().writes(15))
	}
	/// Storage: `Communities::Info` (r:1 w:0)
	/// Proof: `Communities::Info` (`max_values`: None, `max_size`: Some(19), added: 2494, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Account` (r:1 w:1)
	/// Proof: `CommunityMemberships::Account` (`max_values`: None, `max_size`: Some(88), added: 2563, mode: `MaxEncodedLen`)
	/// Storage: `Communities::CommunityRanksSum` (r:1 w:1)
	/// Proof: `Communities::CommunityRanksSum` (`max_values`: None, `max_size`: Some(22), added: 2497, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Attribute` (r:3 w:1)
	/// Proof: `CommunityMemberships::Attribute` (`max_values`: None, `max_size`: Some(479), added: 2954, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::ItemConfigOf` (r:1 w:1)
	/// Proof: `CommunityMemberships::ItemConfigOf` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Collection` (r:1 w:1)
	/// Proof: `CommunityMemberships::Collection` (`max_values`: None, `max_size`: Some(82), added: 2557, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Item` (r:1 w:1)
	/// Proof: `CommunityMemberships::Item` (`max_values`: None, `max_size`: Some(861), added: 3336, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::ItemMetadataOf` (r:1 w:0)
	/// Proof: `CommunityMemberships::ItemMetadataOf` (`max_values`: None, `max_size`: Some(347), added: 2822, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::ItemPriceOf` (r:0 w:1)
	/// Proof: `CommunityMemberships::ItemPriceOf` (`max_values`: None, `max_size`: Some(89), added: 2564, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::ItemAttributesApprovalsOf` (r:0 w:1)
	/// Proof: `CommunityMemberships::ItemAttributesApprovalsOf` (`max_values`: None, `max_size`: Some(1001), added: 3476, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::PendingSwapOf` (r:0 w:1)
	/// Proof: `CommunityMemberships::PendingSwapOf` (`max_values`: None, `max_size`: Some(71), added: 2546, mode: `MaxEncodedLen`)
	fn remove_member() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1086`
		//  Estimated: `9852`
		// Minimum execution time: 226_120_000 picoseconds.
		Weight::from_parts(335_232_000, 0)
			.saturating_add(Weight::from_parts(0, 9852))
			.saturating_add(T::DbWeight::get().reads(11))
			.saturating_add(T::DbWeight::get().writes(10))
	}
	/// Storage: `Communities::Info` (r:1 w:0)
	/// Proof: `Communities::Info` (`max_values`: None, `max_size`: Some(19), added: 2494, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Account` (r:1 w:0)
	/// Proof: `CommunityMemberships::Account` (`max_values`: None, `max_size`: Some(88), added: 2563, mode: `MaxEncodedLen`)
	/// Storage: `Communities::CommunityRanksSum` (r:1 w:1)
	/// Proof: `Communities::CommunityRanksSum` (`max_values`: None, `max_size`: Some(22), added: 2497, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Attribute` (r:1 w:1)
	/// Proof: `CommunityMemberships::Attribute` (`max_values`: None, `max_size`: Some(479), added: 2954, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Collection` (r:1 w:1)
	/// Proof: `CommunityMemberships::Collection` (`max_values`: None, `max_size`: Some(82), added: 2557, mode: `MaxEncodedLen`)
	fn promote_member() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `799`
		//  Estimated: `3944`
		// Minimum execution time: 143_861_000 picoseconds.
		Weight::from_parts(151_550_000, 0)
			.saturating_add(Weight::from_parts(0, 3944))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `Communities::Info` (r:1 w:0)
	/// Proof: `Communities::Info` (`max_values`: None, `max_size`: Some(19), added: 2494, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Account` (r:1 w:0)
	/// Proof: `CommunityMemberships::Account` (`max_values`: None, `max_size`: Some(88), added: 2563, mode: `MaxEncodedLen`)
	/// Storage: `Communities::CommunityRanksSum` (r:1 w:1)
	/// Proof: `Communities::CommunityRanksSum` (`max_values`: None, `max_size`: Some(22), added: 2497, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Attribute` (r:1 w:1)
	/// Proof: `CommunityMemberships::Attribute` (`max_values`: None, `max_size`: Some(479), added: 2954, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Collection` (r:1 w:1)
	/// Proof: `CommunityMemberships::Collection` (`max_values`: None, `max_size`: Some(82), added: 2557, mode: `MaxEncodedLen`)
	fn demote_member() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `841`
		//  Estimated: `3944`
		// Minimum execution time: 108_067_000 picoseconds.
		Weight::from_parts(151_747_000, 0)
			.saturating_add(Weight::from_parts(0, 3944))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `CommunityMemberships::Account` (r:1 w:0)
	/// Proof: `CommunityMemberships::Account` (`max_values`: None, `max_size`: Some(88), added: 2563, mode: `MaxEncodedLen`)
	/// Storage: `CommunityReferenda::ReferendumInfoFor` (r:1 w:1)
	/// Proof: `CommunityReferenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(900), added: 3375, mode: `MaxEncodedLen`)
	/// Storage: `Communities::CommunityDecisionMethod` (r:1 w:0)
	/// Proof: `Communities::CommunityDecisionMethod` (`max_values`: None, `max_size`: Some(36), added: 2511, mode: `MaxEncodedLen`)
	/// Storage: `Communities::CommunityVotes` (r:1 w:1)
	/// Proof: `Communities::CommunityVotes` (`max_values`: None, `max_size`: Some(103), added: 2578, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Attribute` (r:1 w:0)
	/// Proof: `CommunityMemberships::Attribute` (`max_values`: None, `max_size`: Some(479), added: 2954, mode: `MaxEncodedLen`)
	/// Storage: `Scheduler::Agenda` (r:2 w:2)
	/// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(155814), added: 158289, mode: `MaxEncodedLen`)
	fn vote() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2796`
		//  Estimated: `317568`
		// Minimum execution time: 237_455_000 picoseconds.
		Weight::from_parts(308_654_000, 0)
			.saturating_add(Weight::from_parts(0, 317568))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: `CommunityMemberships::Account` (r:1 w:0)
	/// Proof: `CommunityMemberships::Account` (`max_values`: None, `max_size`: Some(88), added: 2563, mode: `MaxEncodedLen`)
	/// Storage: `CommunityReferenda::ReferendumInfoFor` (r:1 w:1)
	/// Proof: `CommunityReferenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(900), added: 3375, mode: `MaxEncodedLen`)
	/// Storage: `Communities::CommunityVotes` (r:1 w:1)
	/// Proof: `Communities::CommunityVotes` (`max_values`: None, `max_size`: Some(103), added: 2578, mode: `MaxEncodedLen`)
	/// Storage: `Communities::CommunityDecisionMethod` (r:1 w:0)
	/// Proof: `Communities::CommunityDecisionMethod` (`max_values`: None, `max_size`: Some(36), added: 2511, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Attribute` (r:1 w:0)
	/// Proof: `CommunityMemberships::Attribute` (`max_values`: None, `max_size`: Some(479), added: 2954, mode: `MaxEncodedLen`)
	fn remove_vote() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2798`
		//  Estimated: `4365`
		// Minimum execution time: 182_794_000 picoseconds.
		Weight::from_parts(236_640_000, 0)
			.saturating_add(Weight::from_parts(0, 4365))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `CommunityReferenda::ReferendumInfoFor` (r:1 w:0)
	/// Proof: `CommunityReferenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(900), added: 3375, mode: `MaxEncodedLen`)
	/// Storage: `Communities::CommunityVotes` (r:1 w:1)
	/// Proof: `Communities::CommunityVotes` (`max_values`: None, `max_size`: Some(103), added: 2578, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Freezes` (r:1 w:1)
	/// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(5682), added: 8157, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Locks` (r:1 w:0)
	/// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
	fn unlock() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1079`
		//  Estimated: `9147`
		// Minimum execution time: 135_798_000 picoseconds.
		Weight::from_parts(175_654_000, 0)
			.saturating_add(Weight::from_parts(0, 9147))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
}
