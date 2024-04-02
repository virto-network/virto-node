use crate::mock::Test;
use crate::weights::{SubstrateWeight, WeightInfo};
use frame_support::weights::{constants::WEIGHT_REF_TIME_PER_NANOS, constants::WEIGHT_REF_TIME_PER_SECOND, Weight};
use sp_runtime::Perbill;

#[test]
fn weights() {
	// max block: 0.5s compute with 12s average block time
	const MAX_BLOCK_REF_TIME: u64 = WEIGHT_REF_TIME_PER_SECOND.saturating_div(2); // https://github.com/paritytech/cumulus/blob/98e68bd54257b4039a5d5b734816f4a1b7c83a9d/parachain-template/runtime/src/lib.rs#L221
	const MAX_BLOCK_POV_SIZE: u64 = 5 * 1024 * 1024; // https://github.com/paritytech/polkadot/blob/ba1f65493d91d4ab1787af2fd6fe880f1da90586/primitives/src/v4/mod.rs#L384
	const MAX_BLOCK_WEIGHT: Weight = Weight::from_parts(MAX_BLOCK_REF_TIME, MAX_BLOCK_POV_SIZE);
	// max extrinsics: 75% of block
	const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75); // https://github.com/paritytech/cumulus/blob/d20c4283fe85df0c1ef8cb7c9eb7c09abbcbfa31/parachain-template/runtime/src/lib.rs#L218
	let max_total_extrinsics = MAX_BLOCK_WEIGHT * NORMAL_DISPATCH_RATIO;
	// max extrinsic: max total extrinsics less average on_initialize ratio and less
	// base extrinsic weight
	const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5); // https://github.com/paritytech/cumulus/blob/d20c4283fe85df0c1ef8cb7c9eb7c09abbcbfa31/parachain-template/runtime/src/lib.rs#L214
	const BASE_EXTRINSIC: Weight = Weight::from_parts(WEIGHT_REF_TIME_PER_NANOS.saturating_mul(125_000), 0); // https://github.com/paritytech/cumulus/blob/d20c4283fe85df0c1ef8cb7c9eb7c09abbcbfa31/parachain-template/runtime/src/weights/extrinsic_weights.rs#L26

	let max_extrinsic_weight = max_total_extrinsics
		.saturating_sub(MAX_BLOCK_WEIGHT * AVERAGE_ON_INITIALIZE_RATIO)
		.saturating_sub(BASE_EXTRINSIC);
	assert_eq!(max_extrinsic_weight, Weight::from_parts(349_875_000_000, 3_670_016));

	println!("max block weight: {MAX_BLOCK_WEIGHT}");
	println!("max total extrinsics weight: {max_total_extrinsics}");
	println!("max extrinsic weight: {max_extrinsic_weight}\n");

	let mut total = Weight::zero();

	for (function, weight) in vec![
		// Examples: call available weight functions with various parameters (as applicable) to gauge weight usage in
		// comparison to limits
		("create", SubstrateWeight::<Test>::create()),
		(
			"set_metadata (64, 256, 63)",
			SubstrateWeight::<Test>::set_metadata(64, 256, 64),
		),
		("set_decision_method", SubstrateWeight::<Test>::set_decision_method()),
		("add_member", SubstrateWeight::<Test>::add_member()),
		("remove_member", SubstrateWeight::<Test>::remove_member()),
		("promote", SubstrateWeight::<Test>::promote()),
		("demote", SubstrateWeight::<Test>::demote()),
		("vote", SubstrateWeight::<Test>::vote()),
		("remove_vote", SubstrateWeight::<Test>::remove_vote()),
		("unlock", SubstrateWeight::<Test>::unlock()),
	] {
		println!("{function}: {weight:?}",);
		println!(
			"\tpercentage of max extrinsic weight: {:.2}% (ref_time), {:.2}% (proof_size)",
			(weight.ref_time() as f64 / max_extrinsic_weight.ref_time() as f64) * 100.0,
			(weight.proof_size() as f64 / max_extrinsic_weight.proof_size() as f64) * 100.0,
		);
		println!(
			"\tmax tx per block: {} (ref_time), {} (proof_size)",
			max_extrinsic_weight.ref_time() / weight.ref_time(),
			max_extrinsic_weight.proof_size() / weight.proof_size().max(1)
		);

		assert!(weight.all_lt(max_extrinsic_weight));

		total += weight;
	}

	// output total weight, useful for evaluating net weight changes when optimising
	println!("\ntotal weight: {total:?}");
}
