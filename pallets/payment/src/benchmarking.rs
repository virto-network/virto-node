#[cfg(feature = "runtime-benchmarks")]
use super::*;
use orml_traits::MultiCurrency;
use frame_benchmarking::{benchmarks, account, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

const SEED: u32 = 0;

  benchmarks! {
    // the longest path in create extrinsic is succesful create
    create {
      let caller: T::AccountId = whitelisted_caller();
      let to: T::AccountId = account("to", 1, SEED);
      T::Tokens::make_free_balance_be(&caller, 100.into());
      const CURRENCY_ID: u32 = 2;
    }: _(RawOrigin::Signed(caller), to, CURRENCY_ID, 1.into())
    verify {
			// the escrow amount should be reserved
      assert_eq!(crate::mock::Tokens::free_balance(CURRENCY_ID, &caller), 80);
      assert_eq!(crate::mock::Tokens::free_balance(CURRENCY_ID, &to), 0);
		}

    // the longest path in release extrinsic is successful release
    release {
      let caller: T::AccountId = whitelisted_caller();
      let to: T::AccountId = account("to", 1, SEED);
      T::Tokens::make_free_balance_be(&caller, 100.into());
      // create escrow
    }: _(RawOrigin::Signed(caller), to)
    verify {
			// the escrow amount should be reserved
      assert_eq!(crate::mock::Tokens::free_balance(CURRENCY_ID, &caller), 80);
      assert_eq!(crate::mock::Tokens::free_balance(CURRENCY_ID, &to), 20);
		}

    // the longest path in cancel extrinsic is a succesful cancel
    cancel {
      let caller: T::AccountId = whitelisted_caller();
      let to: T::AccountId = account("to", 1, SEED);
      T::Tokens::make_free_balance_be(&caller, 100.into());
      const CURRENCY_ID: u32 = 2;
      // create escrow
    }: _(RawOrigin::Signed(caller), to)
    verify {
			// the escrow amount should be reserved
      assert_eq!(crate::mock::Tokens::free_balance(CURRENCY_ID, &caller), 100);
      assert_eq!(crate::mock::Tokens::free_balance(CURRENCY_ID, &to), 0);
		}

    // the longest path in resolve extrinsic is to release or cancel
    resolve {
      let caller: T::AccountId = whitelisted_caller();
      let to: T::AccountId = account("to", 1, SEED);
      T::Tokens::make_free_balance_be(&caller, 100.into());
      const CURRENCY_ID: u32 = 2;
      // create escrow
    }: _(RawOrigin::Signed(caller), to, CURRENCY_ID, 1.into())
    verify {
			// the escrow amount should be reserved
      assert_eq!(crate::mock::Tokens::free_balance(CURRENCY_ID, &caller), 80);
      assert_eq!(crate::mock::Tokens::free_balance(CURRENCY_ID, &to), 0);
		}
  }


  impl_benchmark_test_suite!(Escrow, crate::mock::new_test_ext(), crate::mock::Test);