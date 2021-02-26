use crate::mock::*;
use frame_support::assert_noop;

const ALICE: AccountId = 1;
const BOB: AccountId = 2;
const COIN_A: CurrencyId = 1;
const COIN_B: CurrencyId = 2;

#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Transfers::transfer(Origin::signed(ALICE), COIN_A, COIN_B, BOB, 10),
            crate::Error::<Test>::NotImplemented
        );
    });
}
