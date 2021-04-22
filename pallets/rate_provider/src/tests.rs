use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use vln_primitives::{PaymentMethod, RateProvider};

#[test]
fn test_non_whitelisted_call_must_fail() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Rates::update_price(Origin::signed(1), 1, 2, PaymentMethod::BankX, 1),
            crate::Error::<Test>::NotPermitted
        );
    });
}

#[test]
fn test_add_rates_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(Rates::get_rates(1, 2, PaymentMethod::BankX, 10), None);
        assert_ok!(Rates::update_price(
            Origin::signed(10),
            1,
            2,
            PaymentMethod::BankX,
            1
        ),);
        assert_eq!(Rates::get_rates(1, 2, PaymentMethod::BankX, 10), Some(1));
    });
}

#[test]
fn test_remove_rates_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Rates::update_price(
            Origin::signed(10),
            1,
            2,
            PaymentMethod::BankX,
            1
        ),);
        assert_eq!(Rates::get_rates(1, 2, PaymentMethod::BankX, 10), Some(1));
        assert_ok!(Rates::remove_price(
            Origin::signed(10),
            1,
            2,
            PaymentMethod::BankX
        ),);
        assert_eq!(Rates::get_rates(1, 2, PaymentMethod::BankX, 10), None);
    });
}
