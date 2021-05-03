use crate::mock::*;
use crate::Swaps;
use frame_support::assert_ok;
use orml_traits::MultiCurrency;
use sp_runtime::{FixedU128, Permill};
use vln_primitives::*;

#[test]
fn test_swap_in_lifecycle() {
    new_test_ext().execute_with(|| {
        let base = 1;
        let quote = 2;
        let amount = 10;
        let swap_owner = 1;
        let expected_swap_id = 1;

        // update provider price
        assert_ok!(RatePallet::update_price(
            Origin::signed(PROVIDER_ONE),
            base,
            quote,
            PaymentMethod::BankX,
            Permill::from_percent(1)
        ),);

        // ------------create swap-------------------
        assert_ok!(HumanSwap::create_swap_in(
            Origin::signed(swap_owner),
            base,
            quote,
            PaymentMethod::BankX,
            amount,
            PROVIDER_ONE
        ),);

        assert_eq!(
            Swaps::<Test>::get(swap_owner, expected_swap_id),
            Some(Swap {
                human: PROVIDER_ONE,
                kind: SwapKind::In(SwapIn::Created),
                price: PairPrice {
                    pair: AssetPair { base, quote },
                    price: FixedU128::from(101),
                },
                amount,
            })
        );

        // -------provider accepts swap-----------------
        assert_ok!(HumanSwap::provider_process_swap_in(
            Origin::signed(PROVIDER_ONE),
            swap_owner,
            expected_swap_id,
            SwapIn::Accepted(vec![])
        ),);

        assert_eq!(
            Swaps::<Test>::get(swap_owner, expected_swap_id),
            Some(Swap {
                human: PROVIDER_ONE,
                kind: SwapKind::In(SwapIn::Accepted(vec![])),
                price: PairPrice {
                    pair: AssetPair { base, quote },
                    price: FixedU128::from(101),
                },
                amount,
            })
        );
        // ensure the balances are reserved from provider account
        assert_eq!(Tokens::total_balance(2, &PROVIDER_ONE), 100);
        assert_eq!(Tokens::free_balance(2, &PROVIDER_ONE), 90);

        // -------user confirms swap-----------------
        assert_ok!(HumanSwap::confirm_swap_in(
            Origin::signed(swap_owner),
            expected_swap_id,
            vec![]
        ),);

        assert_eq!(
            Swaps::<Test>::get(swap_owner, expected_swap_id),
            Some(Swap {
                human: PROVIDER_ONE,
                kind: SwapKind::In(SwapIn::Confirmed(vec![])),
                price: PairPrice {
                    pair: AssetPair { base, quote },
                    price: FixedU128::from(101),
                },
                amount,
            })
        );
        // ensure the balances are reserved from provider account
        assert_eq!(Tokens::total_balance(2, &PROVIDER_ONE), 100);
        assert_eq!(Tokens::free_balance(2, &PROVIDER_ONE), 90);

        // -------provider completes swapin-----------------
        assert_ok!(HumanSwap::complete_swap_in(
            Origin::signed(PROVIDER_ONE),
            swap_owner,
            expected_swap_id,
        ),);

        assert_eq!(
            Swaps::<Test>::get(swap_owner, expected_swap_id),
            Some(Swap {
                human: PROVIDER_ONE,
                kind: SwapKind::In(SwapIn::Completed),
                price: PairPrice {
                    pair: AssetPair { base, quote },
                    price: FixedU128::from(101),
                },
                amount,
            })
        );
        // ensure the balances are reserved from provider account
        assert_eq!(Tokens::total_balance(2, &PROVIDER_ONE), 90);
        assert_eq!(Tokens::free_balance(2, &PROVIDER_ONE), 90);
        assert_eq!(Tokens::free_balance(2, &swap_owner), 110);
    });
}

#[test]
fn test_swap_out_lifecycle() {
    new_test_ext().execute_with(|| {
        let base = 1;
        let quote = 2;
        let amount = 10;
        let swap_owner = 1;
        let expected_swap_id = 1;

        // update provider price
        assert_ok!(RatePallet::update_price(
            Origin::signed(PROVIDER_ONE),
            base,
            quote,
            PaymentMethod::BankX,
            Permill::from_percent(1)
        ),);

        //----------------create swap out-----------------
        assert_ok!(HumanSwap::create_swap_out(
            Origin::signed(swap_owner),
            base,
            quote,
            PaymentMethod::BankX,
            amount,
            PROVIDER_ONE
        ),);

        assert_eq!(
            Swaps::<Test>::get(swap_owner, expected_swap_id),
            Some(Swap {
                human: PROVIDER_ONE,
                kind: SwapKind::Out(SwapOut::Created),
                price: PairPrice {
                    pair: AssetPair { base, quote },
                    price: FixedU128::from(101),
                },
                amount,
            })
        );
        // ensure swap owner balances are reserved
        assert_eq!(Tokens::total_balance(2, &swap_owner), 100);
        assert_eq!(Tokens::free_balance(2, &swap_owner), 90);

        //----------------provider confirms swapout-----------------
        assert_ok!(HumanSwap::provider_process_swap_out(
            Origin::signed(PROVIDER_ONE),
            swap_owner,
            expected_swap_id,
            SwapOut::Confirmed(vec![])
        ),);

        assert_eq!(
            Swaps::<Test>::get(swap_owner, expected_swap_id),
            Some(Swap {
                human: PROVIDER_ONE,
                kind: SwapKind::Out(SwapOut::Confirmed(vec![])),
                price: PairPrice {
                    pair: AssetPair { base, quote },
                    price: FixedU128::from(101),
                },
                amount,
            })
        );
        // ensure swap owner balances are reserved
        assert_eq!(Tokens::total_balance(2, &swap_owner), 100);
        assert_eq!(Tokens::free_balance(2, &swap_owner), 90);

        //----------------user completes swapout-----------------
        assert_ok!(HumanSwap::complete_swap_out(
            Origin::signed(swap_owner),
            expected_swap_id,
        ),);

        assert_eq!(
            Swaps::<Test>::get(swap_owner, expected_swap_id),
            Some(Swap {
                human: PROVIDER_ONE,
                kind: SwapKind::Out(SwapOut::Completed),
                price: PairPrice {
                    pair: AssetPair { base, quote },
                    price: FixedU128::from(101),
                },
                amount,
            })
        );
        // ensure swap owner balances are reserved
        assert_eq!(Tokens::total_balance(2, &swap_owner), 90);
        assert_eq!(Tokens::free_balance(2, &swap_owner), 90);
        assert_eq!(Tokens::free_balance(2, &PROVIDER_ONE), 110);
    });
}
