use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
type NativeBalance = <Test as crate::Config>::NativeBalance;
use frame_support::{
	dispatch::GetDispatchInfo,
	traits::{fungible::*, Hooks},
};

fn create_a_remark_call() -> RuntimeCall {
	let bytes = b"hello, world".to_vec();
	let pallet_call = frame_system::Call::<Test>::remark_with_event { remark: bytes.clone() };
	let call: RuntimeCall = pallet_call.into();
	call
}

// This function is intended to be used to consume fast the credits for testing global and user
// limits fn heavy_call() -> RuntimeCall {
// 	let pallet_call = Call::<Test>::fake_function_with_high_weight {};
// 	let call: RuntimeCall = pallet_call.into();
// 	call
// }

fn move_one_era_in_future() {
	let mut n: u64 = 0;
	while n < CONST_BLOCKS_OF_ERA {
		// Finalize
		System::on_finalize(System::block_number());
		// Increase block
		System::set_block_number(System::block_number() + 1);
		// Initialize block
		System::on_finalize(System::block_number());
		// On idle?
		n += 1;
	}
}

/// Lock/Unlock tests
#[test]
fn lock_dat_balance_success() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let amount_to_lock = 50;

		assert_eq!(NativeBalance::total_issuance(), 0);
		assert_ok!(NativeBalance::mint_into(&alice, 100));

		assert_eq!(NativeBalance::total_issuance(), 100);
		assert_eq!(NativeBalance::free_balance(&alice), 100);
		assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));
	});
}

#[test]
fn lock_dat_balance_must_fail() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let amount_to_lock = 6_000;
		let amount_to_mint = 10_000;

		assert_eq!(NativeBalance::total_issuance(), 0);
		assert_ok!(NativeBalance::mint_into(&alice, amount_to_mint));

		assert_eq!(NativeBalance::total_issuance(), 10_000);
		assert_eq!(NativeBalance::free_balance(&alice), 10_000);
		assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));
		assert_noop!(
			FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock),
			Error::<Test>::NotEnoughFreeBalanceToLock
		);
	});
}

#[test]
fn lock_balance_unknown_user_fails() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let bob = 1;
		let amount_to_lock = 150;

		assert_eq!(NativeBalance::total_issuance(), 0);
		assert_ok!(NativeBalance::mint_into(&alice, 100));

		assert_eq!(NativeBalance::total_issuance(), 100);
		assert_eq!(NativeBalance::free_balance(&alice), 100);
		assert_noop!(
			FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(bob), amount_to_lock),
			Error::<Test>::NotEnoughFreeBalanceToLock
		);
	});
}

#[test]
fn relock_dat_balance_success() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let amount_to_lock = 50;
		let amount_to_relock = 50;

		assert_eq!(NativeBalance::total_issuance(), 0);
		assert_ok!(NativeBalance::mint_into(&alice, 100));

		assert_eq!(NativeBalance::total_issuance(), 100);
		assert_eq!(NativeBalance::free_balance(&alice), 100);
		assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));

		assert_ok!(FeeHandler::lock_balance_for_free_tx(
			RuntimeOrigin::signed(alice),
			amount_to_relock
		));
	});
}

#[test]
fn unlock_dat_balance_for_free_tx_success() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let amount_to_lock = 100;
		let amount_to_unlock = 50;

		assert_eq!(NativeBalance::total_issuance(), 0);
		assert_ok!(NativeBalance::mint_into(&alice, 100));

		assert_eq!(NativeBalance::total_issuance(), 100);
		assert_eq!(NativeBalance::free_balance(&alice), 100);
		assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));

		assert_ok!(FeeHandler::unlock_balance(RuntimeOrigin::signed(alice), amount_to_unlock));
	});
}

#[test]
fn unlock_dat_balance_for_free_tx_fails() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let amount_to_lock = 100;
		let amount_to_unlock = 150;

		assert_eq!(NativeBalance::total_issuance(), 0);
		assert_ok!(NativeBalance::mint_into(&alice, 100));

		assert_eq!(NativeBalance::total_issuance(), 100);
		assert_eq!(NativeBalance::free_balance(&alice), 100);
		assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));

		assert_noop!(
			FeeHandler::unlock_balance(RuntimeOrigin::signed(alice), amount_to_unlock),
			Error::<Test>::NotEnoughBalanceToUnlock
		);
	});
}

// Lock -> Move era -> Unlock
#[test]
fn lock_unlock_in_diff_era_works() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let amount_to_lock = 50;
		let amount_to_unlock = amount_to_lock / 2;

		assert_eq!(NativeBalance::total_issuance(), 0);
		assert_ok!(NativeBalance::mint_into(&alice, 100));

		assert_eq!(NativeBalance::total_issuance(), 100);
		assert_eq!(NativeBalance::free_balance(&alice), 100);
		assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));
		move_one_era_in_future();
		assert_ok!(FeeHandler::unlock_balance(RuntimeOrigin::signed(alice), amount_to_unlock));
	});
}

// Lock -> Move era -> Unlock more than locked
#[test]
fn lock_unlock_in_diff_era_fails() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let amount_to_lock = 50;
		let amount_to_unlock = amount_to_lock * 2;

		assert_eq!(NativeBalance::total_issuance(), 0);
		assert_ok!(NativeBalance::mint_into(&alice, 100));

		assert_eq!(NativeBalance::total_issuance(), 100);
		assert_eq!(NativeBalance::free_balance(&alice), 100);
		assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));
		move_one_era_in_future();
		assert_noop!(
			FeeHandler::unlock_balance(RuntimeOrigin::signed(alice), amount_to_unlock),
			Error::<Test>::NotEnoughBalanceToUnlock
		);
	});
}

// Lock -> Lock -> Move era -> Unlock -> Unlock -> Lock
#[test]
fn lock_unlock_many_times_in_diff_era_works() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let amount_to_lock = 50;
		let amount_to_unlock = amount_to_lock / 10;

		assert_eq!(NativeBalance::total_issuance(), 0);
		assert_ok!(NativeBalance::mint_into(&alice, amount_to_lock));

		assert_eq!(NativeBalance::total_issuance(), amount_to_lock);
		assert_eq!(NativeBalance::free_balance(&alice), amount_to_lock);
		assert_ok!(FeeHandler::lock_balance_for_free_tx(
			RuntimeOrigin::signed(alice),
			amount_to_lock / 5
		));
		assert_ok!(FeeHandler::lock_balance_for_free_tx(
			RuntimeOrigin::signed(alice),
			amount_to_lock / 5
		));
		move_one_era_in_future();
		assert_ok!(FeeHandler::unlock_balance(RuntimeOrigin::signed(alice), amount_to_unlock));
		assert_ok!(FeeHandler::unlock_balance(RuntimeOrigin::signed(alice), amount_to_unlock));
	});
}

// Lock -> Lock -> Move era -> Unlock -> Unlock
#[test]
fn lock_unlock_many_times_in_diff_era_fails() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let amount_to_lock = 128;
		let amount_to_unlock = amount_to_lock / 4;

		assert_eq!(NativeBalance::total_issuance(), 0);
		assert_ok!(NativeBalance::mint_into(&alice, amount_to_lock));

		assert_eq!(NativeBalance::total_issuance(), amount_to_lock);
		assert_eq!(NativeBalance::free_balance(&alice), amount_to_lock);
		assert_ok!(FeeHandler::lock_balance_for_free_tx(
			RuntimeOrigin::signed(alice),
			amount_to_lock / 2
		));
		assert_ok!(FeeHandler::lock_balance_for_free_tx(
			RuntimeOrigin::signed(alice),
			amount_to_lock / 2
		));
		move_one_era_in_future();
		assert_ok!(FeeHandler::unlock_balance(RuntimeOrigin::signed(alice), amount_to_unlock));
		assert_noop!(
			FeeHandler::unlock_balance(RuntimeOrigin::signed(alice), amount_to_lock * 2),
			Error::<Test>::NotEnoughBalanceToUnlock
		);
	});
}

/// TX tests
// Lock -> FreeTX
#[test]
fn free_tx_works() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let call = create_a_remark_call();
		let weight = call.get_dispatch_info().weight;
		let amount_to_lock = (weight.ref_time() * 2).into();
		let amount_to_mint = amount_to_lock * 10;

		assert_eq!(NativeBalance::total_issuance(), 0);
		assert_ok!(NativeBalance::mint_into(&alice, amount_to_mint));

		assert_eq!(NativeBalance::total_issuance(), amount_to_mint);
		assert_eq!(NativeBalance::free_balance(&alice), amount_to_mint);
		assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));
		assert_ok!(FeeHandler::free_tx(RuntimeOrigin::signed(alice), Box::new(call)));
	});
}

// Lock inssuficient amount -> FreeTx
#[test]
fn free_tx_not_enough_credits_user_fails() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let call = create_a_remark_call();
		let weight = call.get_dispatch_info().weight;
		let amount_to_lock = (weight.ref_time() - 1).into();
		let amount_to_mint = amount_to_lock * 10;

		assert_eq!(NativeBalance::total_issuance(), 0);
		assert_ok!(NativeBalance::mint_into(&alice, amount_to_mint));

		assert_eq!(NativeBalance::total_issuance(), amount_to_mint);
		assert_eq!(NativeBalance::free_balance(&alice), amount_to_mint);
		assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));
		assert_noop!(
			FeeHandler::free_tx(RuntimeOrigin::signed(alice), Box::new(call)),
			Error::<Test>::NotEnoughCredits
		);
	});
}

// Perform the maximum number of free tx
#[test]
fn many_free_tx_in_single_era_works() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let call = create_a_remark_call();
		let weight = call.get_dispatch_info().weight;
		let max_n_tx = CONST_MAX_CREDIT_USER_CAN_USE_PER_ERA / weight.ref_time();
		let amount_to_lock = (weight.ref_time() * (max_n_tx + 1)).into();
		let amount_to_mint = amount_to_lock * 10;

		assert_eq!(NativeBalance::total_issuance(), 0);
		assert_ok!(NativeBalance::mint_into(&alice, amount_to_mint));

		assert_eq!(NativeBalance::total_issuance(), amount_to_mint);
		assert_eq!(NativeBalance::free_balance(&alice), amount_to_mint);
		assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));
		for _i in 0..max_n_tx {
			assert_ok!(FeeHandler::free_tx(RuntimeOrigin::signed(alice), Box::new(call.clone())));
		}
	});
}

// Get the max number of possible free tx in base of weight and
// CONST_MAX_CREDIT_USER_CAN_USE_PER_ERA Lock the max possible credit to use
// Use the max number of credits
// Try to use an extra free tx above the limit
#[test]
fn many_free_tx_in_single_era_fails() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let call = create_a_remark_call();
		let weight = call.get_dispatch_info().weight;
		let max_n_tx = CONST_MAX_CREDIT_USER_CAN_USE_PER_ERA / weight.ref_time();
		let amount_to_lock = (weight.ref_time() * (max_n_tx + 1)).into();
		let amount_to_mint = amount_to_lock * 10;

		assert_eq!(NativeBalance::total_issuance(), 0);
		assert_ok!(NativeBalance::mint_into(&alice, amount_to_mint));

		assert_eq!(NativeBalance::total_issuance(), amount_to_mint);
		assert_eq!(NativeBalance::free_balance(&alice), amount_to_mint);
		assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));
		for _i in 0..max_n_tx {
			assert_ok!(FeeHandler::free_tx(RuntimeOrigin::signed(alice), Box::new(call.clone())));
		}
		assert_noop!(
			FeeHandler::free_tx(RuntimeOrigin::signed(alice), Box::new(call)),
			Error::<Test>::OverflowMaxUserCredits
		);
	});
}

// Create the maximum number of users who can use their maximum balance in free tx
// Lock the balance
// Use the free tx
// Johan here use the heavy_call() with a fixed value and reach the max global limit in a single era
// #[test]
// fn many_free_tx_in_single_era_fails_max_global_reached() {
// new_test_ext().execute_with(|| {
// 	let call = heavy_call();
// 	let weight = call.get_dispatch_info().weight;
// 	let max_users = 15;
// 	let amount_to_lock: u128 = CONST_MAX_GLOBAL_TOTAL_CREDIT_TO_USE_PER_ERA/(weight.ref_time());
// 	let amount_to_mint = amount_to_lock * 10;

// 	// println!(max_users);
// 	for user in 0..max_users {
// 		assert_ok!(NativeBalance::mint_into(&user, amount_to_mint));
// 		assert_eq!(NativeBalance::free_balance(&user), amount_to_mint);
// 		assert_ok!(FeeHandler::lock_balance_for_free_tx(
// 			RuntimeOrigin::signed(user),
// 			amount_to_lock
// 		));
// 		assert_ok!(FeeHandler::free_tx(RuntimeOrigin::signed(user), Box::new(call.clone())));
// 	}
// 	assert_ok!(NativeBalance::mint_into(&max_users, amount_to_mint));
// 	assert_eq!(NativeBalance::free_balance(&max_users), amount_to_mint);
// 	assert_ok!(FeeHandler::lock_balance_for_free_tx(
// 		RuntimeOrigin::signed(max_users),
// 		amount_to_lock
// 	));
// 	assert_ok!(FeeHandler::free_tx(RuntimeOrigin::signed(max_users), Box::new(call.clone())));
// });
// }

// Create the maximum number of users who can use their maximum balance in free tx
// Lock the balance
// Use the free tx as many times as necessary to reach the max user limit
// #[test]
// fn many_free_tx_in_single_era_fails_max_user_credit_reached() {
// 	todo!()
// new_test_ext().execute_with(|| {
// 	let alice = 0;
// 	let call = create_a_remark_call();
// 	let weight = call.get_dispatch_info().weight;
// 	let amount_to_lock = (weight.ref_time() * 2).into();
// 	let amount_to_mint = amount_to_lock * 10;
// 	assert_ok!(NativeBalance::mint_into(&alice, amount_to_mint));
// 	assert_eq!(NativeBalance::total_issuance(), amount_to_mint);
// 	assert_eq!(NativeBalance::free_balance(&alice), amount_to_mint);
// 	assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));
// 	// First Ok
// 	assert_ok!(FeeHandler::free_tx(RuntimeOrigin::signed(alice), Box::new(call.clone())));
// 	// Second fails
// 	assert_ok!(FeeHandler::free_tx(RuntimeOrigin::signed(alice), Box::new(call.clone())));
// 	// assert_noop!(
// 	// 	FeeHandler::free_tx(RuntimeOrigin::signed(alice), Box::new(call)),
// 	// 	Error::<Test>::OverflowMaxGlobalCredits
// 	// );
// });
// }

// Lock -> FreeTx -> New Era -> FreeTx
// #[test]
// fn many_free_tx_in_different_era_works() {
// 	new_test_ext().execute_with(|| {
// 		let alice = 0;
// 		let amount_to_lock = 80_000_000;
// 		let amount_to_mint = 100_000_000;
// 		let call = create_a_remark_call();

// 		assert_eq!(NativeBalance::total_issuance(), 0);
// 		assert_ok!(NativeBalance::mint_into(&alice, amount_to_mint));

// 		assert_eq!(NativeBalance::total_issuance(), amount_to_mint);
// 		assert_eq!(NativeBalance::free_balance(&alice), amount_to_mint);
// 		assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));
// 		assert_ok!(FeeHandler::free_tx(RuntimeOrigin::signed(alice), Box::new(call)));
// 	});
// }

// Lock big amount -> New era -> Unlock certain amount but locked enough for a call -> FreeTx
// #[test]
// fn lock_unlock_single_free_tx_in_same_era_works() {
// 	new_test_ext().execute_with(|| {
// 		let alice = 0;
// 		let amount_to_lock = 80_000_000;
// 		let amount_to_mint = 100_000_000;
// 		let call = create_a_remark_call();

// 		assert_eq!(NativeBalance::total_issuance(), 0);
// 		assert_ok!(NativeBalance::mint_into(&alice, amount_to_mint));

// 		assert_eq!(NativeBalance::total_issuance(), amount_to_mint);
// 		assert_eq!(NativeBalance::free_balance(&alice), amount_to_mint);
// 		assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));
// 		assert_ok!(FeeHandler::free_tx(RuntimeOrigin::signed(alice), Box::new(call)));
// 	});
// }

// Lock above weight of call -> Unlock without leaving enough for a free call ->  FreeTx
// #[test]
// fn lock_unlock_single_free_tx_in_same_era_fails() {
// 	new_test_ext().execute_with(|| {
// 		let alice = 0;
// 		let amount_to_lock = 80_000_000;
// 		let amount_to_mint = 100_000_000;
// 		let call = create_a_remark_call();

// 		assert_eq!(NativeBalance::total_issuance(), 0);
// 		assert_ok!(NativeBalance::mint_into(&alice, amount_to_mint));

// 		assert_eq!(NativeBalance::total_issuance(), amount_to_mint);
// 		assert_eq!(NativeBalance::free_balance(&alice), amount_to_mint);
// 		assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));
// 		assert_ok!(FeeHandler::free_tx(RuntimeOrigin::signed(alice), Box::new(call)));
// 	});
// }

// Lock a lot -> Unlock -> FreeTX -> Lock - > Unlock -> FreeTX
// #[test]
// fn lock_unlock_multiple_free_tx_in_same_era_works() {
// 	new_test_ext().execute_with(|| {
// 		let alice = 0;
// 		let amount_to_lock = 80_000_000;
// 		let amount_to_mint = 100_000_000;
// 		let call = create_a_remark_call();

// 		assert_eq!(NativeBalance::total_issuance(), 0);
// 		assert_ok!(NativeBalance::mint_into(&alice, amount_to_mint));

// 		assert_eq!(NativeBalance::total_issuance(), amount_to_mint);
// 		assert_eq!(NativeBalance::free_balance(&alice), amount_to_mint);
// 		assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));
// 		assert_ok!(FeeHandler::free_tx(RuntimeOrigin::signed(alice), Box::new(call)));
// 	});
// }

// Lock a lot -> Unlock -> FreeTX -> Lock - > Unlock without leaving enough for a free call ->
// FreeTx #[test]
// fn lock_unlock_multiple_free_tx_in_same_era_fails() {
// 	new_test_ext().execute_with(|| {
// 		let alice = 0;
// 		let amount_to_lock = 80_000_000;
// 		let amount_to_mint = 100_000_000;
// 		let call = create_a_remark_call();

// 		assert_eq!(NativeBalance::total_issuance(), 0);
// 		assert_ok!(NativeBalance::mint_into(&alice, amount_to_mint));

// 		assert_eq!(NativeBalance::total_issuance(), amount_to_mint);
// 		assert_eq!(NativeBalance::free_balance(&alice), amount_to_mint);
// 		assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));
// 		assert_ok!(FeeHandler::free_tx(RuntimeOrigin::signed(alice), Box::new(call)));
// 	});
// }

// Lock -> New era -> Unlock -> Free Tx
// #[test]
// fn lock_unlock_single_free_tx_in_diff_era_works() {
// 	new_test_ext().execute_with(|| {
// 		let alice = 0;
// 		let amount_to_lock = 80_000_000;
// 		let amount_to_mint = 100_000_000;
// 		let call = create_a_remark_call();

// 		assert_eq!(NativeBalance::total_issuance(), 0);
// 		assert_ok!(NativeBalance::mint_into(&alice, amount_to_mint));

// 		assert_eq!(NativeBalance::total_issuance(), amount_to_mint);
// 		assert_eq!(NativeBalance::free_balance(&alice), amount_to_mint);
// 		assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));
// 		assert_ok!(FeeHandler::free_tx(RuntimeOrigin::signed(alice), Box::new(call)));
// 	});
// }

//  Lock -> New era -> Unlock without leaving enough for a free call ->  FreeTx
// #[test]
// fn lock_unlock_single_free_tx_in_diff_era_fails() {
// 	new_test_ext().execute_with(|| {
// 		let alice = 0;
// 		let amount_to_lock = 80_000_000;
// 		let amount_to_mint = 100_000_000;
// 		let call = create_a_remark_call();

// 		assert_eq!(NativeBalance::total_issuance(), 0);
// 		assert_ok!(NativeBalance::mint_into(&alice, amount_to_mint));

// 		assert_eq!(NativeBalance::total_issuance(), amount_to_mint);
// 		assert_eq!(NativeBalance::free_balance(&alice), amount_to_mint);
// 		assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));
// 		assert_ok!(FeeHandler::free_tx(RuntimeOrigin::signed(alice), Box::new(call)));
// 	});
// }

//  Lock -> New era -> Unlock ->  FreeTx -> Lock -> New era -> Unlock ->  FreeTx
// #[test]
// fn lock_unlock_multiple_free_tx_in_diff_era_works() {
// 	new_test_ext().execute_with(|| {
// 		let alice = 0;
// 		let amount_to_lock = 80_000_000;
// 		let amount_to_mint = 100_000_000;
// 		let call = create_a_remark_call();

// 		use frame_support::traits::fungible::*;

// 		assert_eq!(NativeBalance::total_issuance(), 0);
// 		assert_ok!(NativeBalance::mint_into(&alice, amount_to_mint));

// 		assert_eq!(NativeBalance::total_issuance(), amount_to_mint);
// 		assert_eq!(NativeBalance::free_balance(&alice), amount_to_mint);
// 		assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));
// 		assert_ok!(FeeHandler::free_tx(RuntimeOrigin::signed(alice), Box::new(call)));
// 	});
// }

//  Lock -> New era -> Unlock ->  FreeTx -> Lock -> New era -> Unlock without leaving enough for a
// free call ->  FreeTx #[test]
// fn lock_unlock_multiple_free_tx_in_diff_era_fails() {
// 	new_test_ext().execute_with(|| {
// 		let alice = 0;
// 		let amount_to_lock = 80_000_000;
// 		let amount_to_mint = 100_000_000;
// 		let call = create_a_remark_call();

// 		assert_eq!(NativeBalance::total_issuance(), 0);
// 		assert_ok!(NativeBalance::mint_into(&alice, amount_to_mint));

// 		assert_eq!(NativeBalance::total_issuance(), amount_to_mint);
// 		assert_eq!(NativeBalance::free_balance(&alice), amount_to_mint);
// 		assert_ok!(FeeHandler::lock_balance_for_free_tx(RuntimeOrigin::signed(alice), amount_to_lock));
// 		assert_ok!(FeeHandler::free_tx(RuntimeOrigin::signed(alice), Box::new(call)));
// 	});
// }
