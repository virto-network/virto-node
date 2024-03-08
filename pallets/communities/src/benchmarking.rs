//! Benchmarking setup for pallet-communities
#![cfg(feature = "runtime-benchmarks")]
use super::*;

use self::{
	origin::DecisionMethod,
	types::{AccountIdOf, CommunityIdOf, Vote},
	Event, Pallet as Communities,
};
use frame_benchmarking::v2::*;
use frame_support::{
	traits::{fungible::Mutate, OriginTrait},
	BoundedVec,
};
use frame_system::{
	pallet_prelude::{BlockNumberFor, OriginFor, RuntimeCallFor},
	RawOrigin,
};
use sp_runtime::traits::StaticLookup;
use sp_std::{vec, vec::Vec};

fn assert_has_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

fn setup_accounts<T: Config>() -> Result<Vec<AccountIdOf<T>>, BenchmarkError> {
	let size = T::BenchmarkHelper::community_desired_size();
	let accounts = (0..size).map(|i| frame_benchmarking::account("community_benchmarking", i, 0));

	for who in accounts.clone() {
		T::Balances::mint_into(&who, 10u32.into())?;
	}

	Ok(accounts.collect())
}

fn add_member_call<T: Config>() -> RuntimeCallFor<T>
where
	RuntimeCallFor<T>: From<crate::Call<T>>,
{
	let new_member = T::Lookup::unlookup(frame_benchmarking::account("community_benchmarking", 0, 0));
	crate::Call::<T>::add_member { who: new_member }.into()
}

#[benchmarks(
	where
		T: frame_system::Config + crate::Config,
		RuntimeCallFor<T>: From<crate::Call<T>>,
		BlockNumberFor<T>: From<u32>
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create() {
		// setup code
		let id = T::BenchmarkHelper::community_id();
		let origin: T::RuntimeOrigin = T::BenchmarkHelper::community_origin(Default::default());
		let origin = OriginTrait::into_caller(origin);

		#[extrinsic_call]
		_(RawOrigin::Root, origin.clone(), id);

		// verification code
		assert_has_event::<T>(Event::CommunityCreated { id, origin }.into());
	}

	#[benchmark]
	fn set_metadata(n: Linear<1, 64>, d: Linear<1, 256>, u: Linear<1, 256>) -> Result<(), BenchmarkError> {
		// setup code
		let id = T::BenchmarkHelper::community_id();
		let admin_origin: T::RuntimeOrigin = T::BenchmarkHelper::community_origin(Default::default());
		Communities::<T>::create(RawOrigin::Root.into(), admin_origin.into_caller(), id)?;

		let name = Some(BoundedVec::truncate_from(vec![0u8; n as usize]));
		let description = Some(BoundedVec::truncate_from(vec![0u8; d as usize]));
		let url = Some(BoundedVec::truncate_from(vec![0u8; u as usize]));

		#[extrinsic_call]
		_(RawOrigin::Root, id, name.clone(), description.clone(), url.clone());

		// verification code
		assert_has_event::<T>(
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
		let id = T::BenchmarkHelper::community_id();
		let admin_origin: T::RuntimeOrigin = T::BenchmarkHelper::community_origin(Default::default());
		Communities::<T>::create(RawOrigin::Root.into(), admin_origin.into_caller(), id)?;

		#[extrinsic_call]
		_(RawOrigin::Root, id, DecisionMethod::Membership);

		// verification code
		assert_has_event::<T>(Event::DecisionMethodSet { id }.into());

		Ok(())
	}

	#[benchmark]
	fn add_member() -> Result<(), BenchmarkError> {
		// setup code
		let (id, origin): (CommunityIdOf<T>, OriginFor<T>) =
			T::BenchmarkHelper::create_community(RawOrigin::Root.into(), None)?;

		let who: AccountIdOf<T> = frame_benchmarking::account("community_benchmarking", 0, 0);
		let membership_id = T::BenchmarkHelper::new_membership_id(id, 0);

		T::BenchmarkHelper::extend_membership(id, membership_id.clone())?;

		#[extrinsic_call]
		_(origin.into_caller(), T::Lookup::unlookup(who.clone()));

		// verification code
		assert_has_event::<T>(
			Event::MemberAdded {
				who: who.clone(),
				membership_id: membership_id.clone(),
			}
			.into(),
		);
		assert!(Communities::<T>::has_membership(&who, membership_id));

		Ok(())
	}

	// #[benchmark]
	// fn remove_member() -> Result<(), BenchmarkError> {}

	// #[benchmark]
	// fn promote_member() -> Result<(), BenchmarkError> {}

	// #[benchmark]
	// fn demote_member() -> Result<(), BenchmarkError> {}

	#[benchmark]
	fn vote() -> Result<(), BenchmarkError> {
		// setup code
		let (id, origin) = T::BenchmarkHelper::create_community(RawOrigin::Root.into(), None)?;
		let members = T::BenchmarkHelper::setup_members(origin.clone(), id, setup_accounts::<T>()?)?;

		let (who, membership_id) = members
			.first()
			.expect("desired size of community to be equal or greather than 1")
			.clone();

		T::BenchmarkHelper::prepare_track_and_submit_referendum(
			RawOrigin::Signed(who.clone()).into(),
			origin.into_caller(),
			add_member_call::<T>(),
		)?;

		#[extrinsic_call]
		_(
			RawOrigin::Signed(who.clone()),
			membership_id.clone(),
			0u32,
			Vote::Standard(true),
		);

		// verification code
		assert_has_event::<T>(
			Event::VoteCasted {
				who: who.clone(),
				poll_index: 0u32.into(),
				vote: Vote::Standard(true),
			}
			.into(),
		);

		Ok(())
	}

	#[benchmark]
	fn remove_vote() -> Result<(), BenchmarkError> {
		// setup code
		let (id, origin) = T::BenchmarkHelper::create_community(RawOrigin::Root.into(), None)?;
		let members = T::BenchmarkHelper::setup_members(origin.clone(), id, setup_accounts::<T>()?)?;

		let (who, membership_id) = members
			.first()
			.expect("desired size of community to be equal or greather than 1")
			.clone();

		T::BenchmarkHelper::prepare_track_and_submit_referendum(
			RawOrigin::Signed(who.clone()).into(),
			origin.into_caller(),
			add_member_call::<T>(),
		)?;

		Communities::<T>::vote(
			RawOrigin::Signed(who.clone()).into(),
			membership_id.clone(),
			0u32,
			Vote::Standard(true),
		)?;

		#[extrinsic_call]
		_(RawOrigin::Signed(who.clone()), membership_id.clone(), 0u32);

		// verification code
		assert_has_event::<T>(
			Event::VoteRemoved {
				who: who.clone(),
				poll_index: 0u32.into(),
			}
			.into(),
		);

		Ok(())
	}

	// // #[benchmark]
	// // fn unlock_vote() -> Result<(), BenchmarkError> {}

	impl_benchmark_test_suite!(
		Communities,
		crate::tests::mock::new_bench_ext(),
		crate::tests::mock::Test
	);
}
