use super::*;

use sp_core::ConstU128;

use pallet_referenda::{TrackIdOf, TracksInfo};
use parachains_common::kusama::currency::QUID;

pub type KreivoTracksInstance = pallet_referenda_tracks::Instance3;
pub type KreivoReferendaInstance = pallet_referenda::Instance3;

parameter_types! {
	pub const AlarmInterval: BlockNumber = 1;
	pub const SubmissionDeposit: Balance = QUID;
	pub const UndecidingTimeout: BlockNumber = 14 * DAYS;
}

pub struct EnsureOriginToTrack;
impl EnsureOriginWithArg<RuntimeOrigin, TrackIdOf<Runtime, KreivoTracksInstance>> for EnsureOriginToTrack {
	type Success = ();

	fn try_origin(
		o: RuntimeOrigin,
		id: &TrackIdOf<Runtime, KreivoTracksInstance>,
	) -> Result<Self::Success, RuntimeOrigin> {
		let track_id_for_origin: TrackIdOf<Runtime, KreivoTracksInstance> =
			KreivoTracks::track_for(&o.clone().caller).map_err(|_| o.clone())?;
		ensure!(&track_id_for_origin == id, o);

		Ok(())
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin(_: &TrackIdOf<Runtime, KreivoTracksInstance>) -> Result<RuntimeOrigin, ()> {
		Ok(RuntimeOrigin::root())
	}
}

impl pallet_referenda_tracks::Config<KreivoTracksInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type TrackId = u16;
	type MaxTracks = ConstU32<65536>;
	type AdminOrigin = EnsureRoot<AccountId>;
	type UpdateOrigin = EnsureOriginToTrack;
	type WeightInfo = pallet_referenda_tracks::weights::SubstrateWeight<Runtime>;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = KreivoTracksBenchmarkHelper;
}

impl pallet_referenda::Config<KreivoReferendaInstance> for Runtime {
	type WeightInfo = pallet_referenda::weights::SubstrateWeight<Self>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type Scheduler = Scheduler;
	type Currency = Balances;
	// Fellows can submit proposals.
	type SubmitOrigin = pallet_ranked_collective::EnsureMember<Runtime, KreivoCollectiveInstance, 1>;
	// TODO: Define this better
	type CancelOrigin = EnsureRoot<AccountId>;
	// TODO: Define this better
	type KillOrigin = EnsureRoot<AccountId>;
	type Slash = ();
	type Votes = pallet_ranked_collective::Votes;
	type Tally = pallet_ranked_collective::TallyOf<Runtime, KreivoCollectiveInstance>;
	type SubmissionDeposit = ConstU128<0>;
	type MaxQueued = ConstU32<100>;
	type UndecidingTimeout = ConstU32<{ 7 * DAYS }>;
	type AlarmInterval = ConstU32<1>;
	type Tracks = KreivoTracks;
	type Preimages = Preimage;
}

#[cfg(feature = "runtime-benchmarks")]
use sp_runtime::SaturatedConversion;

#[cfg(feature = "runtime-benchmarks")]
pub struct KreivoTracksBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl pallet_referenda_tracks::BenchmarkHelper<Runtime, KreivoTracksInstance> for KreivoTracksBenchmarkHelper {
	fn track_id(id: u32) -> TrackIdOf<Runtime, KreivoTracksInstance> {
		id.saturated_into()
	}
}
