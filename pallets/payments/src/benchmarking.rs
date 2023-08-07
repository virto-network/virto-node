//! Benchmarking setup for pallet-burner

use super::*;

#[allow(unused)]
use crate::Pallet as Burner;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
	// compare to the last event record
	let frame_system::EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

benchmarks! {
	burn_asset {
		let amount: BalanceOf<T> = 1_000_000_000u32.into();
		let burn_from_account: T::AccountId = whitelisted_caller();
		let burn_from_account_lookup = T::Lookup::unlookup(burn_from_account.clone());
	}: _(RawOrigin::Root, burn_from_account_lookup, amount)
	verify {
		assert_last_event::<T>(Event::Burnt { burnt_funds: amount, from: burn_from_account }.into());
	}
}

impl_benchmark_test_suite!(Burner, crate::mock::new_test_ext(), crate::mock::Test,);
