use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{FixedU128, Permill};
use vln_primitives::{PaymentMethod, RateProvider, AssetPair};

#[test]
fn test_non_whitelisted_call_must_fail() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Rates::update_price(
                Origin::signed(1),
                1,
                2,
                PaymentMethod::BankX,
                Permill::from_percent(1)
            ),
            crate::Error::<Test>::NotPermitted
        );
    });
}

#[test]
fn test_add_rates_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(Rates::get_rates(AssetPair { base : 1, quote : 2 }, PaymentMethod::BankX, 10), None);
        assert_ok!(Rates::update_price(
            Origin::signed(10),
            1,
            2,
            PaymentMethod::BankX,
            Permill::from_percent(1)
        ),);
        let rate = Rates::get_rates(AssetPair { base : 1, quote : 2 }, PaymentMethod::BankX, 10).unwrap();
        assert_eq!(
            rate,
            FixedU128::from(101)
        );

        assert_ok!(Rates::update_price(
            Origin::signed(10),
            1,
            2,
            PaymentMethod::BankX,
            Permill::from_percent(45)
        ),);
        let rate = Rates::get_rates(AssetPair { base : 1, quote : 2 }, PaymentMethod::BankX, 10).unwrap();
        assert_eq!(
            rate,
            FixedU128::from(145)
        );

        assert_ok!(Rates::update_price(
            Origin::signed(10),
            1,
            2,
            PaymentMethod::BankX,
            Permill::from_float(0.0496)
        ),);
        let rate = Rates::get_rates(AssetPair { base : 1, quote : 2 }, PaymentMethod::BankX, 10).unwrap();
        assert_eq!(
            rate,
            FixedU128::from_float(104.96)
        );

        assert_ok!(Rates::update_price(
            Origin::signed(10),
            1,
            2,
            PaymentMethod::BankX,
            Permill::from_float(0.999)
        ),);
        let rate = Rates::get_rates(AssetPair { base : 1, quote : 2 }, PaymentMethod::BankX, 10).unwrap();
        assert_eq!(
            rate,
            FixedU128::from_float(199.9)
        );

        assert_ok!(Rates::update_price(
            Origin::signed(10),
            1,
            2,
            PaymentMethod::BankX,
            Permill::from_float(0.0)
        ),);
        let rate = Rates::get_rates(AssetPair { base : 1, quote : 2 }, PaymentMethod::BankX, 10).unwrap();
        assert_eq!(
            rate,
            FixedU128::from(100)
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
            Permill::from_percent(1)
        ),);
        assert_eq!(
            Rates::get_rates(AssetPair { base : 1, quote : 2 }, PaymentMethod::BankX, 10),
            Some(FixedU128::from(101))
        );
        assert_ok!(Rates::remove_price(
            Origin::signed(10),
            1,
            2,
            PaymentMethod::BankX
        ),);
        assert_eq!(Rates::get_rates(AssetPair { base : 1, quote : 2 }, PaymentMethod::BankX, 10), None);
    });
}
