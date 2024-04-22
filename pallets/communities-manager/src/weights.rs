#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_communities.
pub trait WeightInfo {
	fn register() -> Weight;
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
		// Minimum execution time: 117_045_000 picoseconds.
		Weight::from_parts(166_053_000, 0)
			.saturating_add(Weight::from_parts(0, 132561))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(14))
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
		// Minimum execution time: 117_045_000 picoseconds.
		Weight::from_parts(166_053_000, 0)
			.saturating_add(Weight::from_parts(0, 132561))
			.saturating_add(RocksDbWeight::get().reads(7))
			.saturating_add(RocksDbWeight::get().writes(14))
	}
}