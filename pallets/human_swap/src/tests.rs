use crate::mock::*;
use crate::Swaps;
use frame_support::{assert_noop, assert_ok};
use orml_traits::MultiCurrency;
use sp_runtime::{FixedPointNumber, FixedU128, Permill};
use vln_primitives::*;

#[test]
fn it_works_for_swap_in_create() {
    new_test_ext().execute_with(|| {
        // update provider price
        assert_ok!(RatePallet::update_price(
            Origin::signed(PROVIDER_ONE),
            1,
            2,
            PaymentMethod::BankX,
            Permill::from_percent(1)
        ),);

        assert_ok!(HumanSwap::create_swap_in(
            Origin::signed(1),
            1,
            2,
            PaymentMethod::BankX,
            10,
            PROVIDER_ONE
        ),);

        assert_eq!(
            Swaps::<Test>::get(1, 1),
            Some(Swap {
                human: PROVIDER_ONE,
                kind: SwapKind::In(SwapIn::Created),
                price: PairPrice {
                    pair: AssetPair { base: 1, quote: 2 },
                    price: FixedU128::zero(),
                },
                amount: 10,
            })
        );
    });
}

#[test]
fn it_works_for_swap_out_create() {
    new_test_ext().execute_with(|| {
        // update provider price
        assert_ok!(RatePallet::update_price(
            Origin::signed(PROVIDER_ONE),
            1,
            2,
            PaymentMethod::BankX,
            Permill::from_percent(1)
        ),);

        assert_ok!(HumanSwap::create_swap_out(
            Origin::signed(1),
            1,
            2,
            PaymentMethod::BankX,
            10,
            PROVIDER_ONE
        ),);

        assert_eq!(
            Swaps::<Test>::get(1, 1),
            Some(Swap {
                human: PROVIDER_ONE,
                kind: SwapKind::Out(SwapOut::Created),
                price: PairPrice {
                    pair: AssetPair { base: 1, quote: 2 },
                    price: FixedU128::zero(),
                },
                amount: 10,
            })
        );

        assert_eq!(Tokens::total_balance(2, &1), 100);
        assert_eq!(Tokens::free_balance(2, &1), 90);
    });
}
