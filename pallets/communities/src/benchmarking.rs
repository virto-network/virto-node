//! Benchmarking setup for pallet-communities
#![cfg(feature = "runtime-benchmarks")]
use self::types::{CommunityIdOf, PalletsOriginOf};

use super::*;

use crate::{
	origin::{DecisionMethod, RawOrigin as Origin},
	types::AssetIdOf,
	Event, Pallet as Communities,
};
use frame_benchmarking::v2::*;
use frame_support::{traits::OriginTrait, BoundedVec};
use frame_system::{pallet_prelude::OriginFor, RawOrigin};

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn get_community_origin_caller<T: Config>(id: CommunityIdOf<T>) -> PalletsOriginOf<T>
where
	<T as frame_system::Config>::RuntimeOrigin: From<Origin<T>>,
{
	let mut origin = Origin::<T>::new(id);
	origin.with_decision_method(origin::DecisionMethod::Rank);

	<Origin<T> as Into<OriginFor<T>>>::into(origin).into_caller()
}

fn create_community<T: Config>(id: CommunityIdOf<T>, origin: PalletsOriginOf<T>) -> Result<(), BenchmarkError> {
	Communities::<T>::create(RawOrigin::Root.into(), origin, id).map_err(|e| e.into())
}

#[benchmarks(
	where
		<T as frame_system::Config>::RuntimeOrigin: From<Origin<T>>
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create() {
		// setup code
		let id = T::BenchmarkHelper::get_community_id();
		let origin = get_community_origin_caller::<T>(id.clone());

		#[extrinsic_call]
		_(RawOrigin::Root, origin.clone(), id);

		// verification code
		assert_last_event::<T>(Event::CommunityCreated { id, origin }.into());
	}

	#[benchmark]
	fn set_metadata(n: Linear<1, 64>, d: Linear<1, 256>, u: Linear<1, 256>) -> Result<(), BenchmarkError> {
		// setup code
		let id = T::BenchmarkHelper::get_community_id();
		create_community::<T>(id, get_community_origin_caller::<T>(id))?;

		let name = Some(BoundedVec::truncate_from(vec![0u8; n as usize]));
		let description = Some(BoundedVec::truncate_from(vec![0u8; d as usize]));
		let url = Some(BoundedVec::truncate_from(vec![0u8; u as usize]));

		#[extrinsic_call]
		_(RawOrigin::Root, id, name.clone(), description.clone(), url.clone());

		// verification code
		assert_last_event::<T>(
			Event::MetadataSet {
				id,
				name,
				description,
				main_url: url,
			}
			.into(),
		);

		Ok(())
	}

	#[benchmark]
	fn set_decision_method() -> Result<(), BenchmarkError> {
		// setup code
		let id = T::BenchmarkHelper::get_community_id();
		create_community::<T>(id, get_community_origin_caller::<T>(id))?;

		let decision_method = DecisionMethod::<AssetIdOf<T>>::Membership;

		#[extrinsic_call]
		_(RawOrigin::Root, id, decision_method);

		// verification code
		assert_last_event::<T>(Event::DecisionMethodSet { id }.into());

		Ok(())
	}

	// #[benchmark]
	// fn vote () -> Result<(), BenchmarkError> {

	// }

	impl_benchmark_test_suite!(
		Communities,
		crate::tests::mock::new_bench_ext(),
		crate::tests::mock::Test
	);
}
