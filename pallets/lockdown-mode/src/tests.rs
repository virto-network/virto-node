use super::{GenesisConfig, *};
use crate::{mock::*, Error, ACTIVATED, DEACTIVATED};
use frame_support::{assert_noop, assert_ok, traits::Contains};
use pallet_balances::{self, Call as BalancesCall};
use pallet_remark::{self, Call as RemarkCall};

#[test]
fn genesis_config_default() {
	let default_genesis = GenesisConfig::<Test>::default();
	assert_eq!(default_genesis.initial_status, ACTIVATED);
}

#[test]
fn genesis_config_initialized() {
	[ACTIVATED, DEACTIVATED].into_iter().for_each(|expected| {
		new_test_ext(expected).execute_with(|| {
			let lockdown_mode = LockdownModeStatus::<Test>::get();
			assert_eq!(lockdown_mode, expected);
		});
	});
}

#[test]
fn activate_lockdown_mode_works() {
	new_test_ext(DEACTIVATED).execute_with(|| {
		assert_ok!(LockdownMode::activate_lockdown_mode(RuntimeOrigin::root()));

		let lockdown_mode = LockdownModeStatus::<Test>::get();
		assert_eq!(lockdown_mode, ACTIVATED);

		assert_noop!(
			LockdownMode::activate_lockdown_mode(RuntimeOrigin::root(),),
			Error::<Test>::LockdownModeAlreadyActivated
		);
	});
}

#[test]
fn deactivate_lockdown_mode_works() {
	new_test_ext(ACTIVATED).execute_with(|| {
		assert_ok!(LockdownMode::deactivate_lockdown_mode(RuntimeOrigin::root()));

		let lockdown_mode = LockdownModeStatus::<Test>::get();
		assert_eq!(lockdown_mode, DEACTIVATED);

		assert_noop!(
			LockdownMode::deactivate_lockdown_mode(RuntimeOrigin::root(),),
			Error::<Test>::LockdownModeAlreadyDeactivated
		);
	});
}

#[test]
fn call_filtered_in_lockdown_mode() {
	new_test_ext(DEACTIVATED).execute_with(|| {
		assert_ok!(LockdownMode::activate_lockdown_mode(RuntimeOrigin::root()));
		let remark_call = RuntimeCall::Remark(RemarkCall::store { remark: vec![1, 2, 3] });
		let allowed = LockdownMode::contains(&remark_call);
		assert!(!allowed);
	});
}

#[test]
fn call_not_filtered_in_lockdown_mode() {
	new_test_ext(DEACTIVATED).execute_with(|| {
		assert_ok!(LockdownMode::activate_lockdown_mode(RuntimeOrigin::root()));
		let balance_call = RuntimeCall::Balances(BalancesCall::transfer_keep_alive { dest: 1, value: 2 });
		let allowed: bool = LockdownMode::contains(&balance_call);
		assert!(allowed);
	});
}

#[test]
fn call_not_filtered_in_normal_mode() {
	new_test_ext(DEACTIVATED).execute_with(|| {
		let lockdown_mode = LockdownModeStatus::<Test>::get();
		assert_eq!(lockdown_mode, DEACTIVATED);
		let balance_call = RuntimeCall::Balances(BalancesCall::transfer_keep_alive { dest: 1, value: 2 });
		let result: bool = LockdownMode::contains(&balance_call);
		assert!(result);
	});
}
