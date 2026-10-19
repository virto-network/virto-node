//! Benchmarking setup for pallet-communities
use super::*;

use frame_benchmarking::v2::*;

use frame_support::{dispatch::DispatchResult, traits::fungible::Mutate};
use frame_system::RawOrigin;

fn assert_has_event<T: frame_system::Config>(generic_event: T::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_has_event(generic_event);
}

fn setup_account<T: Config>(who: &AccountIdOf<T>) -> DispatchResult
where
	NativeBalanceOf<T>: From<u64>,
{
	let initial_balance: NativeBalanceOf<T> = 1_000_000_000_000_000u64.into();
	T::Balances::mint_into(who, initial_balance)?;
	Ok(())
}

#[benchmarks(
where
	T::RuntimeEvent: From<pallet_communities::Event<T>>,
	NativeBalanceOf<T>: From<u64>,
	CommunityIdOf<T>: One,
	T::MembershipId: From<u32>,
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn register() -> Result<(), BenchmarkError> {
		// setup code
		let first_member: AccountIdOf<T> = account("founder", 0, 0);
		setup_account::<T>(&first_member)?;

		let community_id: CommunityIdOf<T> = One::one();

		#[extrinsic_call]
		_(
			RawOrigin::Root,
			community_id,
			BoundedVec::truncate_from(b"Test Community".into()),
			T::Lookup::unlookup(first_member.clone()),
			None,
			None,
		);

		// verification code
		assert_has_event::<T>(Event::<T>::CommunityRegistered { id: community_id }.into());
		Ok(())
	}

	impl_benchmark_test_suite!(Pallet, sp_io::TestExternalities::new(Default::default()), mock::Test);
}
