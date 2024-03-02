//! Benchmarking setup for pallet-communities
#![cfg(feature = "runtime-benchmarks")]
use super::*;

use self::types::{CommunityIdOf, PalletsOriginOf};

use crate::{
	origin::{DecisionMethod, RawOrigin as Origin},
	types::{AccountIdOf, AssetIdOf, Vote},
	Event, Pallet as Communities,
};
use frame_benchmarking::v2::*;
use frame_support::{
	traits::{fungible::Mutate, schedule::DispatchTime, OriginTrait},
	BoundedVec,
};
use frame_system::{
	pallet_prelude::{BlockNumberFor, OriginFor},
	RawOrigin,
};
use pallet_referenda::{self, BoundedCallOf, Curve, TrackInfo, TrackInfoOf};
use parity_scale_codec::Encode;
use sp_runtime::{
	str_array as s,
	traits::{One, StaticLookup},
	Perbill,
};

fn assert_has_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

fn get_community_origin_caller<T: Config>(id: CommunityIdOf<T>) -> PalletsOriginOf<T>
where
	<T as frame_system::Config>::RuntimeOrigin: From<Origin<T>>,
{
	let mut origin = Origin::<T>::new(id);
	origin.with_decision_method(origin::DecisionMethod::Rank);

	<Origin<T> as Into<OriginFor<T>>>::into(origin).into_caller()
}

fn setup_accounts<T: Config>() -> Result<Vec<AccountIdOf<T>>, BenchmarkError> {
	let size = T::BenchmarkHelper::community_desired_size();
	let accounts = (0..size).map(|i| frame_benchmarking::account("community_benchmarking", i, 0));

	for who in accounts.clone() {
		T::Balances::mint_into(&who, 10u32.into())?;
	}

	Ok(accounts.collect())
}

fn create_community<T: Config>(id: CommunityIdOf<T>, origin: PalletsOriginOf<T>) -> Result<(), BenchmarkError> {
	Communities::<T>::create(RawOrigin::Root.into(), origin, id).map_err(|e| e.into())
}

fn create_track<T>(id: CommunityIdOf<T>, pallet_origin: PalletsOriginOf<T>) -> Result<(), BenchmarkError>
where
	T: Config + pallet_referenda_tracks::Config,
	<T as pallet_referenda_tracks::Config>::TrackId: From<CommunityIdOf<T>>,
	BlockNumberFor<T>: From<u32>,
{
	let info: TrackInfoOf<T> = TrackInfo {
		name: s("Community"),
		max_deciding: 1,
		decision_deposit: 5u32.into(),
		prepare_period: 1u32.into(),
		decision_period: 5u32.into(),
		confirm_period: 1u32.into(),
		min_enactment_period: 1u32.into(),
		min_approval: Curve::LinearDecreasing {
			length: Perbill::from_percent(100),
			floor: Perbill::from_percent(50),
			ceil: Perbill::from_percent(100),
		},
		min_support: Curve::LinearDecreasing {
			length: Perbill::from_percent(100),
			floor: Perbill::from_percent(0),
			ceil: Perbill::from_percent(100),
		},
	};

	pallet_referenda_tracks::Pallet::<T, ()>::insert(RawOrigin::Root.into(), id.into(), info, pallet_origin)?;

	Ok(())
}

fn submit_proposal_to_add_member<T>(
	proposal_origin: PalletsOriginOf<T>,
	proposer: AccountIdOf<T>,
) -> Result<(), BenchmarkError>
where
	T: Config + pallet_referenda::Config + pallet_referenda_tracks::Config,
	<T as pallet_referenda_tracks::Config>::TrackId: From<CommunityIdOf<T>>,
	<T as pallet_referenda::Config>::RuntimeCall: From<crate::Call<T>>,
	BlockNumberFor<T>: From<u32>,
{
	let new_member = T::Lookup::unlookup(frame_benchmarking::account("community_benchmarking_new_member", 0, 0));

	let call: <T as pallet_referenda::Config>::RuntimeCall = crate::Call::<T>::add_member { who: new_member }.into();
	let proposal = BoundedCallOf::<T, ()>::Inline(BoundedVec::truncate_from(call.encode()));
	let enactment_moment = DispatchTime::After(One::one()).into();

	create_track::<T>(T::BenchmarkHelper::get_community_id(), proposal_origin.clone())?;

	pallet_referenda::Pallet::<T, ()>::submit(
		RawOrigin::Signed(proposer.clone()).into(),
		Box::new(proposal_origin.clone()),
		proposal,
		enactment_moment,
	)?;

	pallet_referenda::Pallet::<T, ()>::place_decision_deposit(RawOrigin::Signed(proposer).into(), 0)?;

	Ok(())
}

#[benchmarks(
	where
		T: frame_system::Config + crate::Config + pallet_referenda::Config + pallet_referenda_tracks::Config,
		<T as frame_system::Config>::RuntimeOrigin: From<Origin<T>>,
		<T as pallet_referenda::Config>::RuntimeCall: From<crate::Call<T>>,
		<T as pallet_referenda_tracks::Config>::TrackId: From<CommunityIdOf<T>>,
		BlockNumberFor<T>: From<u32>
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
		assert_has_event::<T>(Event::CommunityCreated { id, origin }.into());
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
		let id = T::BenchmarkHelper::get_community_id();
		create_community::<T>(id, get_community_origin_caller::<T>(id))?;

		let decision_method = DecisionMethod::<AssetIdOf<T>>::Membership;

		#[extrinsic_call]
		_(RawOrigin::Root, id, decision_method);

		// verification code
		assert_has_event::<T>(Event::DecisionMethodSet { id }.into());

		Ok(())
	}

	#[benchmark]
	fn add_member() -> Result<(), BenchmarkError> {
		// setup code
		let id = T::BenchmarkHelper::get_community_id();
		let origin = get_community_origin_caller::<T>(id.clone());
		create_community::<T>(id, origin.clone())?;

		T::BenchmarkHelper::initialize_memberships_collection()?;

		let who: AccountIdOf<T> = frame_benchmarking::account("community_benchmarking", 0, 0);
		let membership_id = T::BenchmarkHelper::new_membership_id(id, 0);

		T::BenchmarkHelper::extend_membership(id, membership_id.clone())?;

		#[extrinsic_call]
		_(origin, T::Lookup::unlookup(who.clone()));

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
		let id = T::BenchmarkHelper::get_community_id();
		let community_origin = get_community_origin_caller::<T>(id);
		create_community::<T>(id, community_origin.clone())?;

		let members = T::BenchmarkHelper::setup_members(id, setup_accounts::<T>()?)?;

		let (who, membership_id) = members
			.first()
			.expect("desired size of community to be equal or greather than 1")
			.clone();

		submit_proposal_to_add_member::<T>(community_origin, who.clone())?;

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
		let id = T::BenchmarkHelper::get_community_id();
		let community_origin = get_community_origin_caller::<T>(id);
		create_community::<T>(id, community_origin.clone())?;

		let members = T::BenchmarkHelper::setup_members(id, setup_accounts::<T>()?)?;

		let (who, membership_id) = members
			.first()
			.expect("desired size of community to be equal or greather than 1")
			.clone();

		submit_proposal_to_add_member::<T>(community_origin, who.clone())?;
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

	// #[benchmark]
	// fn unlock_vote () -> Result<(), BenchmarkError> {}

	impl_benchmark_test_suite!(
		Communities,
		crate::tests::mock::new_bench_ext(),
		crate::tests::mock::Test
	);
}
