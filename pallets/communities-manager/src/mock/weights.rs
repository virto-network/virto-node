use frame_support::{
	parameter_types,
	weights::{
		constants::{WEIGHT_REF_TIME_PER_NANOS, WEIGHT_REF_TIME_PER_SECOND},
		Weight,
	},
};
use sp_runtime::Perbill;

pub const MAX_BLOCK_REF_TIME: u64 = WEIGHT_REF_TIME_PER_SECOND.saturating_div(2); // https://github.com/paritytech/cumulus/blob/98e68bd54257b4039a5d5b734816f4a1b7c83a9d/parachain-template/runtime/src/lib.rs#L221
pub const MAX_BLOCK_POV_SIZE: u64 = 5 * 1024 * 1024; // https://github.com/paritytech/polkadot/blob/ba1f65493d91d4ab1787af2fd6fe880f1da90586/primitives/src/v4/mod.rs#L384
pub const MAX_BLOCK_WEIGHT: Weight = Weight::from_parts(MAX_BLOCK_REF_TIME, MAX_BLOCK_POV_SIZE);
// max extrinsics: 75% of block
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75); // https://github.com/paritytech/cumulus/blob/d20c4283fe85df0c1ef8cb7c9eb7c09abbcbfa31/parachain-template/runtime/src/lib.rs#L218
																	  // max extrinsic: max total extrinsics less average on_initialize ratio and less
																	  // base extrinsic weight
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5); // https://github.com/paritytech/cumulus/blob/d20c4283fe85df0c1ef8cb7c9eb7c09abbcbfa31/parachain-template/runtime/src/lib.rs#L214
pub const BASE_EXTRINSIC: Weight = Weight::from_parts(WEIGHT_REF_TIME_PER_NANOS.saturating_mul(125_000), 0); // https://github.com/paritytech/cumulus/blob/d20c4283fe85df0c1ef8cb7c9eb7c09abbcbfa31/parachain-template/runtime/src/weights/extrinsic_weights.rs#L26

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Weight::from_parts(MAX_BLOCK_REF_TIME, MAX_BLOCK_POV_SIZE);
	pub const MaxScheduledPerBlock: u32 = 512;
}
