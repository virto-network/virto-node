use super::*;

use pallet_referenda::{TrackIdOf, TracksInfo};
use parachains_common::kusama::currency::QUID;

pub type CommunityTracksInstance = pallet_referenda_tracks::Instance2;
pub type CommunityReferendaInstance = pallet_referenda::Instance2;

parameter_types! {
	pub const AlarmInterval: BlockNumber = 1;
	pub const SubmissionDeposit: Balance = 1 * QUID;
	pub const UndecidingTimeout: BlockNumber = 14 * DAYS;
}

pub struct EnsureOriginToTrack;
impl EnsureOriginWithArg<RuntimeOrigin, TrackIdOf<Runtime, CommunityTracksInstance>> for EnsureOriginToTrack {
	type Success = ();

	fn try_origin(
		o: RuntimeOrigin,
		id: &TrackIdOf<Runtime, CommunityTracksInstance>,
	) -> Result<Self::Success, RuntimeOrigin> {
		let track_id_for_origin: TrackIdOf<Runtime, CommunityTracksInstance> =
			CommunityTracks::track_for(&o.clone().caller).map_err(|_| o.clone())?;
		ensure!(&track_id_for_origin == id, o);

		Ok(())
	}
}

impl pallet_referenda_tracks::Config<CommunityTracksInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type TrackId = CommunityId;
	type MaxTracks = ConstU32<65536>;
	type AdminOrigin = EnsureRoot<AccountId>;
	type UpdateOrigin = EnsureOriginToTrack;
	type WeightInfo = pallet_referenda_tracks::weights::SubstrateWeight<Runtime>;
}

// Paritally from https://github.com/polkadot-fellows/runtimes/blob/b5ba0e91d5dd3c4020e848b27be5f2b47e16f281/relay/kusama/src/governance/mod.rs#L75
impl pallet_referenda::Config<CommunityReferendaInstance> for Runtime {
	type WeightInfo = pallet_referenda::weights::SubstrateWeight<Runtime>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type Scheduler = Scheduler;
	type Currency = Balances;
	type SubmitOrigin = frame_system::EnsureSigned<AccountId>;
	type CancelOrigin = EnsureRoot<AccountId>;
	type KillOrigin = EnsureRoot<AccountId>;
	type Slash = Treasury;
	type Votes = pallet_communities::types::VoteWeight;
	type Tally = pallet_communities::types::Tally<Runtime>;
	type SubmissionDeposit = SubmissionDeposit;
	type MaxQueued = ConstU32<100>;
	type UndecidingTimeout = UndecidingTimeout;
	type AlarmInterval = AlarmInterval;
	type Tracks = CommunityTracks;
	type Preimages = Preimage;
}
