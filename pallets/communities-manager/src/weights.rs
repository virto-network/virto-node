#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_communities.
pub trait WeightInfo {
	fn register() -> Weight;
	fn create_memberships(q: u16, ) -> Weight;
}

/// Weights for pallet_communities using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `Communities::Info` (r:1 w:1)
	/// Proof: `Communities::Info` (`max_values`: None, `max_size`: Some(19), added: 2494, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Collection` (r:1 w:1)
	/// Proof: `CommunityMemberships::Collection` (`max_values`: None, `max_size`: Some(82), added: 2557, mode: `MaxEncodedLen`)
	/// Storage: `CommunityTracks::Tracks` (r:1 w:1)
	/// Proof: `CommunityTracks::Tracks` (`max_values`: None, `max_size`: Some(129), added: 2604, mode: `MaxEncodedLen`)
	/// Storage: `CommunityTracks::TracksIds` (r:1 w:1)
	/// Proof: `CommunityTracks::TracksIds` (`max_values`: Some(1), `max_size`: Some(131076), added: 131571, mode: `MaxEncodedLen`)
	/// Storage: `KreivoCollective::Members` (r:1 w:1)
	/// Proof: `KreivoCollective::Members` (`max_values`: None, `max_size`: Some(42), added: 2517, mode: `MaxEncodedLen`)
	/// Storage: `KreivoCollective::MemberCount` (r:1 w:1)
	/// Proof: `KreivoCollective::MemberCount` (`max_values`: None, `max_size`: Some(14), added: 2489, mode: `MaxEncodedLen`)
	/// Storage: `Communities::CommunityIdFor` (r:0 w:1)
	/// Proof: `Communities::CommunityIdFor` (`max_values`: None, `max_size`: Some(622), added: 3097, mode: `MaxEncodedLen`)
	/// Storage: `KreivoCollective::IndexToId` (r:0 w:1)
	/// Proof: `KreivoCollective::IndexToId` (`max_values`: None, `max_size`: Some(54), added: 2529, mode: `MaxEncodedLen`)
	/// Storage: `KreivoCollective::IdToIndex` (r:0 w:1)
	/// Proof: `KreivoCollective::IdToIndex` (`max_values`: None, `max_size`: Some(54), added: 2529, mode: `MaxEncodedLen`)
	/// Storage: `CommunityTracks::OriginToTrackId` (r:0 w:1)
	/// Proof: `CommunityTracks::OriginToTrackId` (`max_values`: None, `max_size`: Some(622), added: 3097, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::CollectionRoleOf` (r:0 w:1)
	/// Proof: `CommunityMemberships::CollectionRoleOf` (`max_values`: None, `max_size`: Some(67), added: 2542, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::CollectionConfigOf` (r:0 w:1)
	/// Proof: `CommunityMemberships::CollectionConfigOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::CollectionAccount` (r:0 w:1)
	/// Proof: `CommunityMemberships::CollectionAccount` (`max_values`: None, `max_size`: Some(66), added: 2541, mode: `MaxEncodedLen`)
	fn register() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `278`
		//  Estimated: `132561`
		// Minimum execution time: 115_705_000 picoseconds.
		Weight::from_parts(140_658_000, 0)
			.saturating_add(Weight::from_parts(0, 132561))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(14))
	}
	/// Storage: `CommunityMemberships::Item` (r:1023 w:1023)
	/// Proof: `CommunityMemberships::Item` (`max_values`: None, `max_size`: Some(859), added: 3334, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Collection` (r:1 w:1)
	/// Proof: `CommunityMemberships::Collection` (`max_values`: None, `max_size`: Some(82), added: 2557, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::CollectionConfigOf` (r:1 w:0)
	/// Proof: `CommunityMemberships::CollectionConfigOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::ItemConfigOf` (r:1023 w:1023)
	/// Proof: `CommunityMemberships::ItemConfigOf` (`max_values`: None, `max_size`: Some(46), added: 2521, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Account` (r:0 w:1023)
	/// Proof: `CommunityMemberships::Account` (`max_values`: None, `max_size`: Some(86), added: 2561, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::ItemPriceOf` (r:0 w:1023)
	/// Proof: `CommunityMemberships::ItemPriceOf` (`max_values`: None, `max_size`: Some(87), added: 2562, mode: `MaxEncodedLen`)
	/// The range of component `q` is `[1, 1024]`.
	fn create_memberships(q: u16, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `368`
		//  Estimated: `3547 + q * (3334 ±0)`
		// Minimum execution time: 87_026_000 picoseconds.
		Weight::from_parts(110_670_000, 0)
			.saturating_add(Weight::from_parts(0, 3547))
			// Standard Error: 331_618
			.saturating_add(Weight::from_parts(83_810_638, 0).saturating_mul(q.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(q.into())))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((4_u64).saturating_mul(q.into())))
			.saturating_add(Weight::from_parts(0, 3334).saturating_mul(q.into()))
	}
}

impl WeightInfo for () {
  /// Storage: `Communities::Info` (r:1 w:1)
	/// Proof: `Communities::Info` (`max_values`: None, `max_size`: Some(19), added: 2494, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Collection` (r:1 w:1)
	/// Proof: `CommunityMemberships::Collection` (`max_values`: None, `max_size`: Some(82), added: 2557, mode: `MaxEncodedLen`)
	/// Storage: `CommunityTracks::Tracks` (r:1 w:1)
	/// Proof: `CommunityTracks::Tracks` (`max_values`: None, `max_size`: Some(129), added: 2604, mode: `MaxEncodedLen`)
	/// Storage: `CommunityTracks::TracksIds` (r:1 w:1)
	/// Proof: `CommunityTracks::TracksIds` (`max_values`: Some(1), `max_size`: Some(131076), added: 131571, mode: `MaxEncodedLen`)
	/// Storage: `KreivoCollective::Members` (r:1 w:1)
	/// Proof: `KreivoCollective::Members` (`max_values`: None, `max_size`: Some(42), added: 2517, mode: `MaxEncodedLen`)
	/// Storage: `KreivoCollective::MemberCount` (r:1 w:1)
	/// Proof: `KreivoCollective::MemberCount` (`max_values`: None, `max_size`: Some(14), added: 2489, mode: `MaxEncodedLen`)
	/// Storage: `Communities::CommunityIdFor` (r:0 w:1)
	/// Proof: `Communities::CommunityIdFor` (`max_values`: None, `max_size`: Some(622), added: 3097, mode: `MaxEncodedLen`)
	/// Storage: `KreivoCollective::IndexToId` (r:0 w:1)
	/// Proof: `KreivoCollective::IndexToId` (`max_values`: None, `max_size`: Some(54), added: 2529, mode: `MaxEncodedLen`)
	/// Storage: `KreivoCollective::IdToIndex` (r:0 w:1)
	/// Proof: `KreivoCollective::IdToIndex` (`max_values`: None, `max_size`: Some(54), added: 2529, mode: `MaxEncodedLen`)
	/// Storage: `CommunityTracks::OriginToTrackId` (r:0 w:1)
	/// Proof: `CommunityTracks::OriginToTrackId` (`max_values`: None, `max_size`: Some(622), added: 3097, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::CollectionRoleOf` (r:0 w:1)
	/// Proof: `CommunityMemberships::CollectionRoleOf` (`max_values`: None, `max_size`: Some(67), added: 2542, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::CollectionConfigOf` (r:0 w:1)
	/// Proof: `CommunityMemberships::CollectionConfigOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::CollectionAccount` (r:0 w:1)
	/// Proof: `CommunityMemberships::CollectionAccount` (`max_values`: None, `max_size`: Some(66), added: 2541, mode: `MaxEncodedLen`)
	fn register() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `278`
		//  Estimated: `132561`
		// Minimum execution time: 115_705_000 picoseconds.
		Weight::from_parts(140_658_000, 0)
			.saturating_add(Weight::from_parts(0, 132561))
			.saturating_add(RocksDbWeight::get().reads(7))
			.saturating_add(RocksDbWeight::get().writes(14))
	}
	/// Storage: `CommunityMemberships::Item` (r:1023 w:1023)
	/// Proof: `CommunityMemberships::Item` (`max_values`: None, `max_size`: Some(859), added: 3334, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Collection` (r:1 w:1)
	/// Proof: `CommunityMemberships::Collection` (`max_values`: None, `max_size`: Some(82), added: 2557, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::CollectionConfigOf` (r:1 w:0)
	/// Proof: `CommunityMemberships::CollectionConfigOf` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::ItemConfigOf` (r:1023 w:1023)
	/// Proof: `CommunityMemberships::ItemConfigOf` (`max_values`: None, `max_size`: Some(46), added: 2521, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::Account` (r:0 w:1023)
	/// Proof: `CommunityMemberships::Account` (`max_values`: None, `max_size`: Some(86), added: 2561, mode: `MaxEncodedLen`)
	/// Storage: `CommunityMemberships::ItemPriceOf` (r:0 w:1023)
	/// Proof: `CommunityMemberships::ItemPriceOf` (`max_values`: None, `max_size`: Some(87), added: 2562, mode: `MaxEncodedLen`)
	/// The range of component `q` is `[1, 1024]`.
	fn create_memberships(q: u16, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `368`
		//  Estimated: `3547 + q * (3334 ±0)`
		// Minimum execution time: 87_026_000 picoseconds.
		Weight::from_parts(110_670_000, 0)
			.saturating_add(Weight::from_parts(0, 3547))
			// Standard Error: 331_618
			.saturating_add(Weight::from_parts(83_810_638, 0).saturating_mul(q.into()))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().reads((2_u64).saturating_mul(q.into())))
			.saturating_add(RocksDbWeight::get().writes(1))
			.saturating_add(RocksDbWeight::get().writes((4_u64).saturating_mul(q.into())))
			.saturating_add(Weight::from_parts(0, 3334).saturating_mul(q.into()))
	}
}