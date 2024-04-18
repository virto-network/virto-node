//! Benchmarking setup for pallet-communities
#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::v2::*;

use self::Pallet as CommunitiesManager;

use frame_system::RawOrigin;
use pallet_referenda::Curve;
use sp_runtime::{str_array as s, traits::StaticLookup, Perbill};

type RuntimeEventFor<T> = <T as Config>::RuntimeEvent;

fn assert_has_event<T: Config>(generic_event: RuntimeEventFor<T>) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

#[benchmarks(
where
	RuntimeEventFor<T>: From<pallet_communities::Event<T>>,
	BlockNumberFor<T>: From<u32>,
	CommunityIdOf<T>: From<u16>,
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn register() {
		// setup code
		let community_id: CommunityIdOf<T> = 1.into();
		let first_member: AccountIdOf<T> = frame_benchmarking::account("founder", 0, 0);
		let admin_origin: RuntimeOriginFor<T> = frame_system::Origin::<T>::Signed(first_member.clone()).into();
		let admin_origin_caller: PalletsOriginOf<T> = admin_origin.into_caller();

		#[extrinsic_call]
		_(
			RawOrigin::Root,
			community_id,
			TrackInfo {
				name: s("Test Track"),
				max_deciding: 1,
				decision_deposit: 0u32.into(),
				prepare_period: 1u32.into(),
				decision_period: 2u32.into(),
				confirm_period: 1u32.into(),
				min_enactment_period: 1u32.into(),
				min_approval: pallet_referenda::Curve::LinearDecreasing {
					length: Perbill::one(),
					floor: Perbill::zero(),
					ceil: Perbill::one(),
				},
				min_support: Curve::LinearDecreasing {
					length: Perbill::one(),
					floor: Perbill::zero(),
					ceil: Perbill::one(),
				},
			},
			Some(admin_origin_caller.clone()),
			None,
			Some(T::Lookup::unlookup(first_member)),
		);

		// verification code
		assert_has_event::<T>(
			pallet_communities::Event::<T>::CommunityCreated {
				id: community_id,
				origin: admin_origin_caller,
			}
			.into(),
		);
		assert_has_event::<T>(crate::Event::<T>::CommunityRegistered { id: community_id }.into());
	}

	impl_benchmark_test_suite!(
		CommunitiesManager,
		sp_io::TestExternalities::new(Default::default()),
		crate::mock::Test
	);
}
