use super::*;

use frame_benchmarking::v2::BenchmarkError;
use frame_system::pallet_prelude::{OriginFor, RuntimeCallFor};
use sp_runtime::SaturatedConversion;

use pallet_communities::{CommunityIdOf, MembershipIdOf, PollIndexOf};
use pallet_referenda::PalletsOriginOf;
use pallet_referenda_tracks::TrackIdOf;

pub struct TracksBenchmarkHelper;

impl pallet_referenda_tracks::BenchmarkHelper<Test> for TracksBenchmarkHelper {
	fn track_id(id: u32) -> TrackIdOf<Test, ()> {
		id.saturated_into()
	}
}

pub struct CommunityBenchmarkHelper;

impl pallet_communities::BenchmarkHelper<Test> for CommunityBenchmarkHelper {
	fn community_id() -> CommunityIdOf<Test> {
		1
	}

	fn initialize_memberships_collection() -> Result<(), frame_benchmarking::BenchmarkError> {
		unimplemented!()
	}

	fn issue_membership(
		_: CommunityIdOf<Test>,
		_: MembershipIdOf<Test>,
	) -> Result<(), frame_benchmarking::BenchmarkError> {
		unimplemented!()
	}

	fn prepare_track(_: pallet_communities::PalletsOriginOf<Test>) -> Result<(), BenchmarkError> {
		unimplemented!()
	}

	fn prepare_poll(
		_: OriginFor<Test>,
		_: PalletsOriginOf<Test>,
		_: RuntimeCallFor<Test>,
	) -> Result<PollIndexOf<Test>, BenchmarkError> {
		unimplemented!()
	}

	fn finish_poll(_: PollIndexOf<Test>) -> Result<(), BenchmarkError> {
		unimplemented!()
	}
}
