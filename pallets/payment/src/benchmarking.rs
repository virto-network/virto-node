use super::*;

use crate::Pallet as Payment;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use virto_primitives::PaymentState;

const SEED: u32 = 0;

fn setup_caller<T: Config>() -> T::AccountId {
	let caller = whitelisted_caller();
	let value: BalanceOf<T> = 10u32.into();
	let asset: AssetIdOf<T> = "test";
	let _ = T::Asset::deposit(asset, &caller, value);
	caller
}

fn get_asset_and_balance<T: Config>(asset_id: u32, balance: u32) -> (AssetIdOf<T>, BalanceOf<T>) {
	(asset_id.into(), balance.into())
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	// compare to the last event record
	let frame_system::EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

benchmarks! {
	// create a new payment succesfully
	create {
		let caller: T::AccountId = setup_caller::<T>();
		let recipent = account("recipient", 0, SEED);
		let (asset, amount) = get_asset_and_balance::<T>(1,5);
	}: _(RawOrigin::Signed(caller), recipent, asset, amount)
	verify {
		assert_last_event::<T>(Event::<T>::PaymentCreated(caller, asset, amount).into());
	}

	// release an existing payment succesfully
	release {
		let caller: T::AccountId = setup_caller::<T>();
		let recipent = account("recipient", 0, SEED);
		let (asset, amount) = get_asset_and_balance::<T>(1,5);
		Payment::<T>::create(RawOrigin::Signed(caller.clone()).into(), recipent, asset, amount);
	}: _(RawOrigin::Signed(caller), recipent)
	verify {
		assert_last_event::<T>(Event::<T>::PaymentReleased(caller, recipent).into());
	}

	// cancel an existing payment succesfully
	cancel {
		let caller: T::AccountId = setup_caller::<T>();
		let recipent = account("recipient", 0, SEED);
		let (asset, amount) = get_asset_and_balance::<T>(1,5);
		Payment::<T>::create(RawOrigin::Signed(caller.clone()).into(), recipent, asset, amount);
	}: _(RawOrigin::Signed(caller), recipent)
	verify {
		assert_last_event::<T>(Event::<T>::PaymentCancelled(caller, recipent).into());
	}

	// resolve an existing payment succesfully - cancel since that is the most complex route
	resolve {
		let caller: T::AccountId = setup_caller::<T>();
		let recipent = account("recipient", 0, SEED);
		let (asset, amount) = get_asset_and_balance::<T>(1,5);
		Payment::<T>::create(RawOrigin::Signed(caller.clone()).into(), recipent, asset, amount);
	}: _(RawOrigin::Signed(caller), caller, recipent, PaymentState::Cancelled)
	verify {
		assert_last_event::<T>(Event::<T>::PaymentCancelled(caller, recipent).into());
	}
}

impl_benchmark_test_suite!(Payment, crate::mock::new_test_ext(), crate::mock::Test,);
