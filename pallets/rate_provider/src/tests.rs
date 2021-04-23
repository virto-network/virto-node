use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::Percent;
use vln_primitives::{PaymentMethod, RateProvider};

#[test]
fn test_non_whitelisted_call_must_fail() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Rates::update_price(
                Origin::signed(1),
                1,
                2,
                PaymentMethod::BankX,
                Percent::from_parts(1)
            ),
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
            Percent::from_percent(1)
        ),);
        assert_eq!(
            Rates::get_rates(1, 2, PaymentMethod::BankX, 10),
            Some((100, Percent::from_percent(1)))
        );

        assert_ok!(Rates::update_price(
            Origin::signed(10),
            1,
            2,
            PaymentMethod::BankX,
            Percent::from_percent(4)
        ),);
        assert_eq!(
            Rates::get_rates(1, 2, PaymentMethod::BankX, 10),
            Some((100, Percent::from_percent(4)))
        );
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
            Percent::from_parts(1)
        ),);
        assert_eq!(
            Rates::get_rates(1, 2, PaymentMethod::BankX, 10),
            Some((100, Percent::from_percent(1)))
        );
        assert_ok!(Rates::remove_price(
            Origin::signed(10),
            1,
            2,
            PaymentMethod::BankX
        ),);
        assert_eq!(Rates::get_rates(1, 2, PaymentMethod::BankX, 10), None);
    });
}
