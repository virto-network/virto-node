use crate::mock::*;
use crate::primitives::Share;
use frame_support::{assert_ok, traits::Currency};

const ALICE: AccountId = 1;
const BOB: AccountId = 2;
const COIN_A: CurrencyId = 1;
const COIN_B: CurrencyId = 2;

#[test]
fn total_issuance() {
    new_test_with_accounts(&[(ALICE, COIN_A, 100), (BOB, COIN_B, 100)]).execute_with(|| {
        assert_ok!(Asset::mint(Origin::signed(ALICE), COIN_A, 42));
        assert_eq!(Asset::total_issuance(), 42);

        assert_ok!(Asset::mint(Origin::signed(BOB), COIN_B, 58));
        assert_eq!(Asset::total_issuance(), 100);
    });
}

#[test]
fn account_collateral_share() {
    new_test_with_accounts(&[(BOB, COIN_A, 100), (BOB, COIN_B, 100)]).execute_with(|| {
        assert_ok!(Asset::mint(Origin::signed(BOB), COIN_A, 80));
        assert_ok!(Asset::mint(Origin::signed(BOB), COIN_B, 20));

        assert_eq!(
            Asset::account_share(BOB, COIN_A),
            Some(Share::from_percent(80))
        );
        assert_eq!(
            Asset::account_share(BOB, COIN_B),
            Some(Share::from_percent(20))
        );
    });
}
