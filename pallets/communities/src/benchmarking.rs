//! Benchmarking setup for pallet-communities
#![cfg(feature = "runtime-benchmarks")]
use self::types::{CommunityIdOf, PalletsOriginOf};

use super::*;

use crate::{origin::RawOrigin as Origin, Event, Pallet as Communities};
use frame_benchmarking::v2::*;
use frame_support::traits::OriginTrait;
use frame_system::{RawOrigin, pallet_prelude::OriginFor};
use virto_common::CommunityId;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn get_community_origin_caller<T: Config>() -> PalletsOriginOf<T> 
where 
	CommunityIdOf<T>: From<CommunityId>,
	<T as frame_system::Config>::RuntimeOrigin: From<Origin<T>>
{
	<Origin<T> as Into<OriginFor<T>>>::into(Origin::<T>::new(T::BenchmarkHelper::get_community_id())).into_caller()
}

#[benchmarks(
	where 
		CommunityIdOf<T>: From<CommunityId>,
		<T as frame_system::Config>::RuntimeOrigin: From<Origin<T>>
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create() {
		// setup code
		let id = T::BenchmarkHelper::get_community_id();
		let origin: <<T as frame_system::Config>::RuntimeOrigin as OriginTrait>::PalletsOrigin = get_community_origin_caller::<T>();

		#[extrinsic_call]
		_(RawOrigin::Root, origin.clone(), id);

		// verification code
		assert_last_event::<T>(Event::CommunityCreated { id, origin }.into());
	}

	// #[benchmark]
	// fn set_metadata(n: Linear<1, 64>, d: Linear<1, 256>, u: Linear<1, 64>) -> Result<(), BenchmarkError> {

	// }

	impl_benchmark_test_suite!(
		Communities,
		crate::tests::mock::new_bench_ext(),
		crate::tests::mock::Test
	);
}
