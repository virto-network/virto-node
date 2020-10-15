use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::BadOrigin;

const ROOT: u64 = 1;
const ASSET: u32 = 1;

#[test]
fn attest_increases_asset_supply() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::add_member(Origin::signed(ROOT), 2));
        assert_ok!(Membership::add_member(Origin::signed(ROOT), 3));

        assert_ok!(TestProvider::attest(Origin::signed(2), ASSET, 123));
        assert_ok!(TestProvider::attest(Origin::signed(3), ASSET, 456));
        assert_eq!(Tokens::total_issuance(ASSET), 579);
    });
}

#[test]
fn only_providers_can_attest() {
    new_test_ext().execute_with(|| {
        // only "root" can register
        assert_noop!(Membership::add_member(Origin::signed(2), 2), BadOrigin);
        assert_noop!(
            TestProvider::attest(Origin::signed(2), ASSET, 123),
            Error::<Test>::NotAProvider
        );
        assert_ok!(Membership::add_member(Origin::signed(ROOT), 2));
        assert_ok!(TestProvider::attest(Origin::signed(2), ASSET, 123));
    });
}
