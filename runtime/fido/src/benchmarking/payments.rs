use crate::{AccountId, Assets, Payment, Runtime, Sudo};
use frame_benchmarking::{account, whitelisted_caller};
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use orml_payments::BoundedDataOf;
use orml_traits::MultiCurrency;
use sp_runtime::Percent;
use sp_std::vec;
use virto_primitives::Asset;

const SEED: u32 = 0;
const INITIAL_AMOUNT: u128 = 100;
const SOME_AMOUNT: u128 = 80;
const MAX_REMARK_LENGTH: u8 = 50;

fn get_currency_id() -> Asset {
	Asset::default()
}

runtime_benchmarks! {
	{ Runtime, orml_payments }

	// create a new payment with remark sucessfully
	pay {
		let caller = whitelisted_caller();
		let _ = Assets::deposit(get_currency_id(), &caller, INITIAL_AMOUNT);
		let recipent : AccountId = account("recipient", 0, SEED);
		let x in 1..MAX_REMARK_LENGTH.into();
		let remark : BoundedDataOf<Runtime> = vec![u8::MAX; x.try_into().unwrap()].try_into().unwrap();
	}: _(RawOrigin::Signed(caller.clone()), recipent.clone(), get_currency_id(), SOME_AMOUNT, Some(remark))
	verify {
		assert_eq!(Assets::total_balance(get_currency_id(), &recipent), SOME_AMOUNT);
	}

	// release an existing payment succesfully
	release {
		let caller = whitelisted_caller();
		let _ = Assets::deposit(get_currency_id(), &caller, INITIAL_AMOUNT);
		let recipent : AccountId = account("recipient", 0, SEED);
		Payment::pay(RawOrigin::Signed(caller.clone()).into(), recipent.clone(), get_currency_id(), SOME_AMOUNT, None)?;
	}: _(RawOrigin::Signed(caller.clone()), recipent.clone())
	verify {
		assert_eq!(Assets::free_balance(get_currency_id(), &recipent), SOME_AMOUNT);
	}

	// cancel an existing payment succesfully
	cancel {
		let caller = whitelisted_caller();
		let _ = Assets::deposit(get_currency_id(), &caller, INITIAL_AMOUNT);
		let recipent : AccountId = account("recipient", 0, SEED);
		Payment::pay(RawOrigin::Signed(caller.clone()).into(), recipent.clone(), get_currency_id(), SOME_AMOUNT, None)?;
	}: _(RawOrigin::Signed(recipent.clone()), caller)
	verify {
		assert_eq!(Assets::free_balance(get_currency_id(), &recipent), 0);
	}

	// resolve an existing payment to cancellation - this is the most complex path
	resolve_payment {
		let caller = whitelisted_caller();
		let _ = Assets::deposit(get_currency_id(), &caller, INITIAL_AMOUNT);
		let recipent : AccountId = account("recipient", 0, SEED);
		Payment::pay(RawOrigin::Signed(caller.clone()).into(), recipent.clone(), get_currency_id(), SOME_AMOUNT, None)?;
	}: _(RawOrigin::Signed(Sudo::key().expect("Sudo key not set!")), caller, recipent.clone(), Percent::from_percent(100))
	verify {
		assert_eq!(Assets::free_balance(get_currency_id(), &recipent), 80);
	}

	// creator of payment creates a refund request
	request_refund {
		let caller = whitelisted_caller();
		let _ = Assets::deposit(get_currency_id(), &caller, INITIAL_AMOUNT);
		let recipent : AccountId = account("recipient", 0, SEED);
		Payment::pay(RawOrigin::Signed(caller.clone()).into(), recipent.clone(), get_currency_id(), SOME_AMOUNT, None)?;
	}: _(RawOrigin::Signed(caller.clone()), recipent.clone())
	verify {
		assert_eq!(Assets::free_balance(get_currency_id(), &recipent), 0);
	}

	// recipient of a payment can dispute a refund request
	dispute_refund {
		let caller = whitelisted_caller();
		let _ = Assets::deposit(get_currency_id(), &caller, INITIAL_AMOUNT);
		let recipent : AccountId = account("recipient", 0, SEED);
		Payment::pay(RawOrigin::Signed(caller.clone()).into(), recipent.clone(), get_currency_id(), SOME_AMOUNT, None)?;
		Payment::request_refund(RawOrigin::Signed(caller.clone()).into(), recipent.clone())?;
	}: _(RawOrigin::Signed(recipent.clone()), caller)
	verify {
		assert_eq!(Assets::free_balance(get_currency_id(), &recipent), 0);
	}

	// recipient of a payment can create a payment reqest
	request_payment {
		let caller : AccountId = whitelisted_caller();
		let sender : AccountId = account("recipient", 0, SEED);
	}: _(RawOrigin::Signed(caller.clone()), sender, get_currency_id(), SOME_AMOUNT)
	verify {
	}

	// payment request can be completed by the sender
	accept_and_pay {
		let sender : AccountId = whitelisted_caller();
		let receiver : AccountId = account("recipient", 0, SEED);
		let _ = Assets::deposit(get_currency_id(), &sender, INITIAL_AMOUNT);
		Payment::request_payment(RawOrigin::Signed(receiver.clone()).into(), sender.clone(), get_currency_id(), SOME_AMOUNT)?;
	}: _(RawOrigin::Signed(sender.clone()), receiver.clone())
	verify {
		assert_eq!(Assets::free_balance(get_currency_id(), &receiver), SOME_AMOUNT);
	}

}
