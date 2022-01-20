use super::*;

use crate::{Pallet as Payment, PaymentState, Payment as PaymentStore};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;

const SEED: u32 = 0;
const CURRENCY_ID: u32 = 1u32;
const INITIAL_AMOUNT: u32 = 100;
const SOME_AMOUNT: u32 = 80;
const RESOLVER_ACCOUNT: u8 = 10;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	// compare to the last event record
	let frame_system::EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

benchmarks! {
	where_clause { where T::Asset: MultiCurrency<
		<T as frame_system::Config>::AccountId,
		CurrencyId = u32, Balance = u32
	>
}
	// create a new payment succesfully
	pay {
		let caller = whitelisted_caller();
		let _ = T::Asset::deposit(CURRENCY_ID, &caller, INITIAL_AMOUNT);
		let recipent = account("recipient", 0, SEED);
	}: _(RawOrigin::Signed(caller.clone()), recipent, CURRENCY_ID, SOME_AMOUNT)
	verify {
		assert_last_event::<T>(Event::<T>::PaymentCreated { from : caller, asset: CURRENCY_ID, amount: SOME_AMOUNT}.into());
	}

	// create a new payment with remark sucessfully
	pay_with_remark {
		let caller = whitelisted_caller();
		let _ = T::Asset::deposit(CURRENCY_ID, &caller, INITIAL_AMOUNT);
		let recipent = account("recipient", 0, SEED);
	}: _(RawOrigin::Signed(caller.clone()), recipent, CURRENCY_ID, SOME_AMOUNT, "test".into())
	verify {
		assert_last_event::<T>(Event::<T>::PaymentCreated { from: caller, asset: CURRENCY_ID, amount: SOME_AMOUNT}.into());
	}

	// release an existing payment succesfully
	release {
		let caller = whitelisted_caller();
		let _ = T::Asset::deposit(CURRENCY_ID, &caller, INITIAL_AMOUNT);
		let recipent : T::AccountId = account("recipient", 0, SEED);
		Payment::<T>::pay(RawOrigin::Signed(caller.clone()).into(), recipent.clone(), CURRENCY_ID, SOME_AMOUNT)?;
	}: _(RawOrigin::Signed(caller.clone()), recipent.clone())
	verify {
		assert_last_event::<T>(Event::<T>::PaymentReleased { from: caller, to: recipent}.into());
	}

	// cancel an existing payment succesfully
	cancel {
		let caller = whitelisted_caller();
		let _ = T::Asset::deposit(CURRENCY_ID, &caller, INITIAL_AMOUNT);
		let recipent : T::AccountId = account("recipient", 0, SEED);
		Payment::<T>::pay(RawOrigin::Signed(caller.clone()).into(), recipent.clone(), CURRENCY_ID, SOME_AMOUNT)?;
	}: _(RawOrigin::Signed(recipent.clone()), caller.clone())
	verify {
		assert_last_event::<T>(Event::<T>::PaymentCancelled { from: caller, to: recipent}.into());
	}

	// resolve an existing payment succesfully - cancel since that is the most complex route
	resolve_cancel_payment {
		let caller = whitelisted_caller();
		let _ = T::Asset::deposit(CURRENCY_ID, &caller, INITIAL_AMOUNT);
		let recipent : T::AccountId = account("recipient", 0, SEED);
		Payment::<T>::pay(RawOrigin::Signed(caller.clone()).into(), recipent.clone(), CURRENCY_ID, SOME_AMOUNT)?;
		let resolver = PaymentStore::<T>::get(caller.clone(), recipent.clone()).unwrap().resolver_account;
	}: _(RawOrigin::Signed(resolver), caller.clone(), recipent.clone())
	verify {
		assert_last_event::<T>(Event::<T>::PaymentCancelled { from: caller, to: recipent}.into());
	}
}

impl_benchmark_test_suite!(Payment, crate::mock::new_test_ext(), crate::mock::Test,);
