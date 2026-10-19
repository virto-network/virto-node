#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_communities.
pub trait WeightInfo {
	fn register() -> Weight;
	fn issue_memberships(n: u32, ) -> Weight;
	fn retire_memberships(n: u32, ) -> Weight;
	fn complete_retirement(a: u32, ) -> Weight;
}

/// Weights for pallet_communities using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `Communities::Info` (r:1 w:1)
	/// Proof: `Communities::Info` (`max_values`: None, `max_size`: Some(19), added: 2494, mode: `MaxEncodedLen`)
	/// Storage: `Communities::CommunityIdFor` (r:1 w:1)
	/// Proof: `Communities::CommunityIdFor` (`max_values`: None, `max_size`: Some(622), added: 3097, mode: `MaxEncodedLen`)
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
		// Minimum execution time: 172_021_000 picoseconds.
		Weight::from_parts(181_851_000, 0)
			.saturating_add(Weight::from_parts(0, 132561))
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(14))
			// Hand-estimated, until the benchmark replaces it: a registration someone pays for makes two
			// transfers and a hold, and writes the record.
			.saturating_add(Weight::from_parts(60_000_000, 10_000))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	/// Hand-estimated, until `/cmd bench --pallet pallet_communities_manager` replaces it.
	///
	/// Per membership: the manager's mint, lock and index (the item, its config, its owner and the
	/// collection), the hold on the community account, and the deposit record.
	/// The range of component `n` is `[0, 1024]`.
	fn issue_memberships(n: u32, ) -> Weight {
		Weight::from_parts(60_000_000, 0)
			.saturating_add(Weight::from_parts(0, 8_000))
			.saturating_add(Weight::from_parts(160_000_000, 0).saturating_mul(n.into()))
			.saturating_add(Weight::from_parts(0, 9_000).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().reads((10_u64).saturating_mul(n.into())))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((11_u64).saturating_mul(n.into())))
	}
	/// Hand-estimated, until `/cmd bench --pallet pallet_communities_manager` replaces it.
	///
	/// Per membership: the deposit record, the manager's move to the retirement holder, and the
	/// retirement record.
	/// The range of component `n` is `[0, 1024]`.
	fn retire_memberships(n: u32, ) -> Weight {
		Weight::from_parts(40_000_000, 0)
			.saturating_add(Weight::from_parts(0, 4_000))
			.saturating_add(Weight::from_parts(120_000_000, 0).saturating_mul(n.into()))
			.saturating_add(Weight::from_parts(0, 7_000).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((6_u64).saturating_mul(n.into())))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((5_u64).saturating_mul(n.into())))
	}
	/// Hand-estimated, until `/cmd bench --pallet pallet_communities_manager` replaces it.
	///
	///
	/// The burn and the release of the hold, plus the clearing of each of the `a` attributes.
	fn complete_retirement(a: u32, ) -> Weight {
		Weight::from_parts(200_000_000, 0)
			.saturating_add(Weight::from_parts(60_000_000, 0).saturating_mul(a.into()))
			.saturating_add(Weight::from_parts(0, 20_000))
			.saturating_add(Weight::from_parts(0, 3_000).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().writes(8))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(a.into())))
	}
}

impl WeightInfo for () {
  /// Storage: `Communities::Info` (r:1 w:1)
	/// Proof: `Communities::Info` (`max_values`: None, `max_size`: Some(19), added: 2494, mode: `MaxEncodedLen`)
	/// Storage: `Communities::CommunityIdFor` (r:1 w:1)
	/// Proof: `Communities::CommunityIdFor` (`max_values`: None, `max_size`: Some(622), added: 3097, mode: `MaxEncodedLen`)
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
		// Minimum execution time: 172_021_000 picoseconds.
		Weight::from_parts(181_851_000, 0)
			.saturating_add(Weight::from_parts(0, 132561))
			.saturating_add(RocksDbWeight::get().reads(8))
			.saturating_add(RocksDbWeight::get().writes(14))
			// Hand-estimated, until the benchmark replaces it: a registration someone pays for makes two
			// transfers and a hold, and writes the record.
			.saturating_add(Weight::from_parts(60_000_000, 10_000))
			.saturating_add(RocksDbWeight::get().reads(6))
			.saturating_add(RocksDbWeight::get().writes(7))
	}
	/// Hand-estimated, until `/cmd bench --pallet pallet_communities_manager` replaces it.
	///
	/// Per membership: the manager's mint, lock and index (the item, its config, its owner and the
	/// collection), the hold on the community account, and the deposit record.
	/// The range of component `n` is `[0, 1024]`.
	fn issue_memberships(n: u32, ) -> Weight {
		Weight::from_parts(60_000_000, 0)
			.saturating_add(Weight::from_parts(0, 8_000))
			.saturating_add(Weight::from_parts(160_000_000, 0).saturating_mul(n.into()))
			.saturating_add(Weight::from_parts(0, 9_000).saturating_mul(n.into()))
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().reads((10_u64).saturating_mul(n.into())))
			.saturating_add(RocksDbWeight::get().writes(1))
			.saturating_add(RocksDbWeight::get().writes((11_u64).saturating_mul(n.into())))
	}
	/// Hand-estimated, until `/cmd bench --pallet pallet_communities_manager` replaces it.
	///
	/// Per membership: the deposit record, the manager's move to the retirement holder, and the
	/// retirement record.
	/// The range of component `n` is `[0, 1024]`.
	fn retire_memberships(n: u32, ) -> Weight {
		Weight::from_parts(40_000_000, 0)
			.saturating_add(Weight::from_parts(0, 4_000))
			.saturating_add(Weight::from_parts(120_000_000, 0).saturating_mul(n.into()))
			.saturating_add(Weight::from_parts(0, 7_000).saturating_mul(n.into()))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().reads((6_u64).saturating_mul(n.into())))
			.saturating_add(RocksDbWeight::get().writes(1))
			.saturating_add(RocksDbWeight::get().writes((5_u64).saturating_mul(n.into())))
	}
	/// Hand-estimated, until `/cmd bench --pallet pallet_communities_manager` replaces it.
	///
	///
	/// The burn and the release of the hold, plus the clearing of each of the `a` attributes.
	fn complete_retirement(a: u32, ) -> Weight {
		Weight::from_parts(200_000_000, 0)
			.saturating_add(Weight::from_parts(60_000_000, 0).saturating_mul(a.into()))
			.saturating_add(Weight::from_parts(0, 20_000))
			.saturating_add(Weight::from_parts(0, 3_000).saturating_mul(a.into()))
			.saturating_add(RocksDbWeight::get().reads(8))
			.saturating_add(RocksDbWeight::get().reads((2_u64).saturating_mul(a.into())))
			.saturating_add(RocksDbWeight::get().writes(8))
			.saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(a.into())))
	}
}