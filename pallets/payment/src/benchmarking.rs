use super::*;

use crate::{Pallet as Payment, Payment as PaymentStore};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::{OnFinalize, OnInitialize};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::traits::One;
use sp_std::vec;
use virto_primitives::Asset;

const SEED: u32 = 0;
const INITIAL_AMOUNT: u128 = 100;
const SOME_AMOUNT: u128 = 80;
const MAX_REMARK_LENGTH: u8 = 50;

fn get_currency_id() -> Asset {
	Asset::default()
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	// compare to the last event record
	let frame_system::EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

pub fn run_to_block<T: Config>(n: T::BlockNumber) {
	while frame_system::Pallet::<T>::block_number() < n {
		frame_system::Pallet::<T>::on_finalize(frame_system::Pallet::<T>::block_number());
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + One::one(),
		);
		frame_system::Pallet::<T>::on_initialize(frame_system::Pallet::<T>::block_number());
	}
}

benchmarks! {
	where_clause { where T::Asset: MultiCurrency<
		<T as frame_system::Config>::AccountId,
		CurrencyId = Asset, Balance = u128
	>
}

	// create a new payment with remark sucessfully
	pay {
		let caller = whitelisted_caller();
		let _ = T::Asset::deposit(get_currency_id(), &caller, INITIAL_AMOUNT);
		let recipent = account("recipient", 0, SEED);
		let x in 1..MAX_REMARK_LENGTH.into();
		let remark : BoundedDataOf<T> = vec![u8::MAX; x.try_into().unwrap()].try_into().unwrap();
	}: _(RawOrigin::Signed(caller.clone()), recipent, get_currency_id(), SOME_AMOUNT, Some(remark.clone()))
	verify {
		assert_last_event::<T>(Event::<T>::PaymentCreated { from: caller, asset: get_currency_id(), amount: SOME_AMOUNT, remark: Some(remark)}.into());
	}

	// release an existing payment succesfully
	release {
		let caller = whitelisted_caller();
		let _ = T::Asset::deposit(get_currency_id(), &caller, INITIAL_AMOUNT);
		let recipent : T::AccountId = account("recipient", 0, SEED);
		Payment::<T>::pay(RawOrigin::Signed(caller.clone()).into(), recipent.clone(), get_currency_id(), SOME_AMOUNT, None)?;
	}: _(RawOrigin::Signed(caller.clone()), recipent.clone())
	verify {
		assert_last_event::<T>(Event::<T>::PaymentReleased { from: caller, to: recipent}.into());
	}

	// cancel an existing payment succesfully
	cancel {
		let caller = whitelisted_caller();
		let _ = T::Asset::deposit(get_currency_id(), &caller, INITIAL_AMOUNT);
		let recipent : T::AccountId = account("recipient", 0, SEED);
		Payment::<T>::pay(RawOrigin::Signed(caller.clone()).into(), recipent.clone(), get_currency_id(), SOME_AMOUNT, None)?;
	}: _(RawOrigin::Signed(recipent.clone()), caller.clone())
	verify {
		assert_last_event::<T>(Event::<T>::PaymentCancelled { from: caller, to: recipent}.into());
	}

	// resolve an existing payment to cancellation
	resolve_cancel_payment {
		let caller = whitelisted_caller();
		let _ = T::Asset::deposit(get_currency_id(), &caller, INITIAL_AMOUNT);
		let recipent : T::AccountId = account("recipient", 0, SEED);
		Payment::<T>::pay(RawOrigin::Signed(caller.clone()).into(), recipent.clone(), get_currency_id(), SOME_AMOUNT, None)?;
		let resolver = PaymentStore::<T>::get(caller.clone(), recipent.clone()).unwrap().resolver_account;
	}: _(RawOrigin::Signed(resolver), caller.clone(), recipent.clone())
	verify {
		assert_last_event::<T>(Event::<T>::PaymentCancelled { from: caller, to: recipent}.into());
	}

	// resolve an existing payment to release
	resolve_release_payment {
		let caller = whitelisted_caller();
		let _ = T::Asset::deposit(get_currency_id(), &caller, INITIAL_AMOUNT);
		let recipent : T::AccountId = account("recipient", 0, SEED);
		Payment::<T>::pay(RawOrigin::Signed(caller.clone()).into(), recipent.clone(), get_currency_id(), SOME_AMOUNT, None)?;
		let resolver = PaymentStore::<T>::get(caller.clone(), recipent.clone()).unwrap().resolver_account;
	}: _(RawOrigin::Signed(resolver), caller.clone(), recipent.clone())
	verify {
		assert_last_event::<T>(Event::<T>::PaymentReleased { from: caller, to: recipent}.into());
	}

	// creator of payment creates a refund request
	request_refund {
		let caller = whitelisted_caller();
		let _ = T::Asset::deposit(get_currency_id(), &caller, INITIAL_AMOUNT);
		let recipent : T::AccountId = account("recipient", 0, SEED);
		Payment::<T>::pay(RawOrigin::Signed(caller.clone()).into(), recipent.clone(), get_currency_id(), SOME_AMOUNT, None)?;
	}: _(RawOrigin::Signed(caller.clone()), recipent.clone())
	verify {
		assert_last_event::<T>(Event::<T>::PaymentCreatorRequestedRefund { from: caller, to: recipent, expiry: 601u32.into() }.into());
	}

	// recipient of a payment can dispute a refund request
	dispute_refund {
		let caller = whitelisted_caller();
		let _ = T::Asset::deposit(get_currency_id(), &caller, INITIAL_AMOUNT);
		let recipent : T::AccountId = account("recipient", 0, SEED);
		Payment::<T>::pay(RawOrigin::Signed(caller.clone()).into(), recipent.clone(), get_currency_id(), SOME_AMOUNT, None)?;
		Payment::<T>::request_refund(RawOrigin::Signed(caller.clone()).into(), recipent.clone())?;
	}: _(RawOrigin::Signed(recipent.clone()), caller.clone())
	verify {
		assert_last_event::<T>(Event::<T>::PaymentRefundDisputed { from: caller, to: recipent}.into());
	}

	// recipient of a payment can create a payment reqest
	request_payment {
		let caller : T::AccountId = whitelisted_caller();
		let sender : T::AccountId = account("recipient", 0, SEED);
	}: _(RawOrigin::Signed(caller.clone()), sender.clone(), get_currency_id(), SOME_AMOUNT)
	verify {
		assert_last_event::<T>(Event::<T>::PaymentRequestCreated { from: sender, to: caller}.into());
	}

	// payment request can be completed by the sender
	accept_and_pay {
		let sender : T::AccountId = whitelisted_caller();
		let receiver : T::AccountId = account("recipient", 0, SEED);
		let _ = T::Asset::deposit(get_currency_id(), &sender, INITIAL_AMOUNT);
		Payment::<T>::request_payment(RawOrigin::Signed(receiver.clone()).into(), sender.clone(), get_currency_id(), SOME_AMOUNT)?;
	}: _(RawOrigin::Signed(sender.clone()), receiver.clone())
	verify {
		assert_last_event::<T>(Event::<T>::PaymentRequestCompleted { from: sender, to: receiver}.into());
	}
}

impl_benchmark_test_suite!(Payment, crate::mock::new_test_ext(), crate::mock::Test,);
