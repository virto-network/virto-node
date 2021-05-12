use crate::mock::*;
use crate::Rates;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{FixedU128, Permill};
use sp_std::collections::btree_map::BTreeMap;
use std::iter::FromIterator;
use vln_primitives::{
    AssetPair, PaymentMethod, RateDetail, RateFixedType, RatePremiumType, RateProvider,
};

#[test]
fn test_non_whitelisted_call_must_fail() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            RatePallet::update_price(
                Origin::signed(1),
                1,
                2,
                PaymentMethod::BankX,
                RateDetail::Premium(Permill::from_percent(1))
            ),
            crate::Error::<Test>::NotPermitted
        );
    });
}

#[test]
fn test_add_rates_work() {
    test_execute_update_price(
        PROVIDER_ONE,
        1,
        2,
        RateDetail::Premium(Permill::from_percent(1)),
        FixedU128::from(101),
        BTreeMap::from_iter(vec![(
            PROVIDER_ONE,
            RateDetail::Premium(Permill::from_percent(1)),
        )]),
    );

    test_execute_update_price(
        PROVIDER_ONE,
        1,
        2,
        RateDetail::Premium(Permill::from_percent(45)),
        FixedU128::from(145),
        BTreeMap::from_iter(vec![(
            PROVIDER_ONE,
            RateDetail::Premium(Permill::from_percent(45)),
        )]),
    );

    test_execute_update_price(
        PROVIDER_ONE,
        1,
        2,
        RateDetail::Premium(Permill::from_float(0.0496)),
        FixedU128::from_float(104.96),
        BTreeMap::from_iter(vec![(
            PROVIDER_ONE,
            RateDetail::Premium(Permill::from_float(0.0496)),
        )]),
    );

    test_execute_update_price(
        PROVIDER_ONE,
        1,
        2,
        RateDetail::Premium(Permill::from_float(0.999)),
        FixedU128::from_float(199.9),
        BTreeMap::from_iter(vec![(
            PROVIDER_ONE,
            RateDetail::Premium(Permill::from_float(0.999)),
        )]),
    );

    test_execute_update_price(
        PROVIDER_TWO,
        1,
        2,
        RateDetail::Premium(Permill::from_percent(0)),
        FixedU128::from(100),
        BTreeMap::from_iter(vec![(
            PROVIDER_TWO,
            RateDetail::Premium(Permill::from_percent(0)),
        )]),
    );

    test_execute_update_price(
        PROVIDER_TWO,
        1,
        2,
        RateDetail::Fixed(FixedU128::from(1001)),
        FixedU128::from(1001),
        BTreeMap::from_iter(vec![(
            PROVIDER_TWO,
            RateDetail::Fixed(FixedU128::from(1001)),
        )]),
    );
}

#[test]
fn test_remove_rates_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(RatePallet::update_price(
            Origin::signed(PROVIDER_ONE),
            1,
            2,
            PaymentMethod::BankX,
            RateDetail::Premium(Permill::from_percent(1))
        ),);
        assert_eq!(
            RatePallet::get_rates(AssetPair { base: 1, quote: 2 }, PaymentMethod::BankX, 10),
            Some(FixedU128::from(101))
        );
        assert_ok!(RatePallet::remove_price(
            Origin::signed(PROVIDER_ONE),
            1,
            2,
            PaymentMethod::BankX
        ),);
        assert_eq!(
            Rates::<Test>::get(AssetPair { base: 1, quote: 2 }, PaymentMethod::BankX,),
            BTreeMap::new()
        );
        assert_eq!(
            RatePallet::get_rates(AssetPair { base: 1, quote: 2 }, PaymentMethod::BankX, 10),
            None
        );
    })
}

fn test_execute_update_price(
    origin: u8,
    base: u32,
    quote: u32,
    premium: RateDetail<RateFixedType, RatePremiumType>,
    rate_result: FixedU128,
    pair_result: BTreeMap<u8, RateDetail<RateFixedType, RatePremiumType>>,
) {
    new_test_ext().execute_with(|| {
        assert_ok!(RatePallet::update_price(
            Origin::signed(origin),
            base,
            quote,
            PaymentMethod::BankX,
            premium
        ),);
        assert_eq!(
            RatePallet::get_rates(AssetPair { base, quote }, PaymentMethod::BankX, origin),
            Some(rate_result)
        );
        assert_eq!(
            Rates::<Test>::get(AssetPair { base, quote }, PaymentMethod::BankX,),
            pair_result
        );
    });
}
