//! Benchmarking setup for pallet-communities
#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::v2::*;

use crate::Pallet as CommunitiesManager;

#[benchmarks]
mod benchmarks {
	use super::*;

	// #[benchmark]
	// fn register() {
	// 	// setup code

	//   #[extrinsic_call]
	// 	_();

	// 	// verification code
	// }

	impl_benchmark_test_suite!(
		CommunitiesManager,
		sp_io::TestExternalities::new(Default::default()),
		crate::mock::Test
	);
}
