//! Benchmarking setup for pallet-asset-registry

use super::*;

#[allow(unused)]
use crate::Pallet as Pass;
use frame_benchmarking::benchmarks;
use frame_support::assert_ok;
use frame_system::RawOrigin;

benchmarks! {
	impl_benchmark_test_suite!(Pass, crate::mock::new_test_ext(), crate::mock::Test);
}
