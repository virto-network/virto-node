use crate::mock::*;
use crate::RateStore;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{FixedU128, Permill};
use sp_std::collections::btree_map::BTreeMap;
use std::iter::FromIterator;
use vln_primitives::{
    AssetPair, PaymentMethod, RateDetail, RatePremiumType, RateProvider, Rates as RatesType,
};

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
    test_execute_update_price(
        10,
        1,
        2,
        Permill::from_percent(1),
        FixedU128::from(101),
        BTreeMap::from_iter(vec![(
            10,
            RateDetail {
                rate: Permill::from_percent(1),
            },
        )]),
    );

    test_execute_update_price(
        10,
        1,
        2,
        Permill::from_percent(45),
        FixedU128::from(145),
        BTreeMap::from_iter(vec![(
            10,
            RateDetail {
                rate: Permill::from_percent(45),
            },
        )]),
    );

    test_execute_update_price(
        10,
        1,
        2,
        Permill::from_float(0.0496),
        FixedU128::from_float(104.96),
        BTreeMap::from_iter(vec![(
            10,
            RateDetail {
                rate: Permill::from_float(0.0496),
            },
        )]),
    );

    test_execute_update_price(
        10,
        1,
        2,
        Permill::from_float(0.999),
        FixedU128::from_float(199.9),
        BTreeMap::from_iter(vec![(
            10,
            RateDetail {
                rate: Permill::from_float(0.999),
            },
        )]),
    );

    test_execute_update_price(
        11,
        1,
        2,
        Permill::from_float(0.0),
        FixedU128::from(100),
        BTreeMap::from_iter(vec![(
            11,
            RateDetail {
                rate: Permill::from_percent(0),
            },
        )]),
    );
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
            Rates::get_rates(AssetPair { base: 1, quote: 2 }, PaymentMethod::BankX, 10),
            Some(FixedU128::from(101))
        );
        assert_ok!(Rates::remove_price(
            Origin::signed(10),
            1,
            2,
            PaymentMethod::BankX
        ),);
        assert_eq!(
            RateStore::<Test>::get(RatesType {
                pair: AssetPair { base: 1, quote: 2 },
                medium: PaymentMethod::BankX,
            }),
            BTreeMap::new()
        );
        assert_eq!(
            Rates::get_rates(AssetPair { base: 1, quote: 2 }, PaymentMethod::BankX, 10),
            None
        );
    });
}

fn test_execute_update_price(
    origin: u8,
    base: u32,
    quote: u32,
    premium: Permill,
    rate_result: FixedU128,
    pair_result: BTreeMap<u8, RateDetail<RatePremiumType>>,
) {
    new_test_ext().execute_with(|| {
        assert_ok!(Rates::update_price(
            Origin::signed(origin),
            base,
            quote,
            PaymentMethod::BankX,
            premium
        ),);
        assert_eq!(
            Rates::get_rates(AssetPair { base, quote }, PaymentMethod::BankX, origin),
            Some(rate_result)
        );
        assert_eq!(
            RateStore::<Test>::get(RatesType {
                pair: AssetPair { base, quote },
                medium: PaymentMethod::BankX,
            }),
            pair_result
        );
    });
}
