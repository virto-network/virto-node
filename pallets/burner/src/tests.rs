use crate::mock::*;
use frame_support::assert_ok;
use frame_system::RawOrigin;

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		let current_balance = Balances::free_balance(1);
		let burn_amount: u64 = 25000000;
		assert_ok!(Burner::burn_asset(RawOrigin::Root.into(), 1, burn_amount));
		let after_balance = Balances::free_balance(1);
		println!("current_balance: {current_balance:?}, after_balance: {after_balance:?}",);
		assert_eq!(after_balance, current_balance - burn_amount);
	});
}
