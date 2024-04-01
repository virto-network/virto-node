use super::*;
use crate::weights::{SubstrateWeight, WeightInfo};
use frame_support::weights::Weight;

#[test]
fn weights() {
	let max_total_extrinsics = MAX_BLOCK_WEIGHT * NORMAL_DISPATCH_RATIO;
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
