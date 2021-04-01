use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use orml_traits::MultiCurrency;

const ALICE: AccountId = 1;
const BOB: AccountId = 2;
const COIN_A: CurrencyId = 1;
const COIN_B: CurrencyId = 2;

#[test]
fn it_works_for_same_asset_transfer() {
    new_test_ext().execute_with(|| {
        assert_ok!(Transfers::transfer(
            Origin::signed(ALICE),
            COIN_A,
            COIN_A,
            BOB,
            10
        ));
        assert_eq!(Tokens::total_issuance(COIN_A), 10);
        assert_eq!(Tokens::total_balance(COIN_A, &BOB), 10);
        assert_eq!(Tokens::free_balance(COIN_A, &BOB), 10);
    });
}

#[test]
fn not_implemented_diff_asset_transfer() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Transfers::transfer(Origin::signed(ALICE), COIN_A, COIN_B, BOB, 10),
            crate::Error::<Test>::NotImplemented
        );
    });
}
