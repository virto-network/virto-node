use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use orml_traits::MultiCurrency;

#[test]
fn whitelisted_attester_mints_and_locks_asset() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::attest(Origin::signed(10), (), 42));
        assert_eq!(Tokens::total_issuance(()), 42);
        assert_eq!(Tokens::total_balance((), &10), 42);
    });
}

#[test]
fn cant_transfer_attested_asset() {
    new_test_ext().execute_with(|| {
        let attester = 10;
        let asset = ();
        assert_ok!(Assets::attest(Origin::signed(attester), asset, 42));

        assert_noop!(
            Tokens::withdraw(asset, &attester, 42),
            orml_tokens::Error::<Test>::LiquidityRestrictions,
        );
        assert_eq!(Tokens::free_balance(asset, &attester), 42);
    });
}

#[test]
fn non_whitelisted_cant_attest() {
    new_test_ext().execute_with(|| {
        let attester = 1;
        let asset = ();
        assert_noop!(
            Assets::attest(Origin::signed(attester), asset, 42),
            crate::Error::<Test>::NotInWhitelist,
        );
    });
}
