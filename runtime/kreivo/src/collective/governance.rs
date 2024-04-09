use super::*;

use pallet_communities::origin::AsSignedByCommunity;
use pallet_ranked_collective::{EnsureMember, TallyOf, Votes};
use parachains_common::kusama::currency::QUID;
use sp_core::ConstU128;

pub type KreivoTracksInstance = pallet_referenda_tracks::Instance1;
pub type KreivoReferendaInstance = pallet_referenda::Instance1;

parameter_types! {
	pub const AlarmInterval: BlockNumber = 1;
	pub const SubmissionDeposit: Balance = QUID;
	pub const UndecidingTimeout: BlockNumber = 14 * DAYS;
}

impl pallet_referenda::Config<KreivoReferendaInstance> for Runtime {
	type WeightInfo = pallet_referenda::weights::SubstrateWeight<Self>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type Scheduler = Scheduler;
	type Currency = Balances;
	// Communities can submit proposals.
	type SubmitOrigin =
		AsEnsureOriginWithArg<EitherOf<EnsureMember<Runtime, KreivoCollectiveInstance, 1>, AsSignedByCommunity<Self>>>;
	type CancelOrigin = EnsureRoot<AccountId>;
	type KillOrigin = EnsureRoot<AccountId>;
	type Slash = ();
	type Votes = Votes;
	type Tally = TallyOf<Runtime, KreivoCollectiveInstance>;
	type SubmissionDeposit = ConstU128<0>;
	type MaxQueued = ConstU32<10>;
	type UndecidingTimeout = ConstU32<{ 2 * DAYS }>;
	type AlarmInterval = ConstU32<1>;
	type Tracks = tracks::TracksInfo;
	type Preimages = Preimage;
}
