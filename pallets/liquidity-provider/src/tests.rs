use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::BadOrigin;
use valiu_node_commons::ValiuCurrencies;

const BTC: ValiuCurrencies = ValiuCurrencies::Btc;
const ROOT: u64 = 1;

#[test]
fn attest_increases_asset_supply() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::add_member(Origin::signed(ROOT), 2));
        assert_ok!(Membership::add_member(Origin::signed(ROOT), 3));

        assert_ok!(TestProvider::attest(Origin::signed(2), BTC, 123));
        assert_ok!(TestProvider::attest(Origin::signed(3), BTC, 456));
        assert_eq!(Tokens::total_issuance(BTC), 579);
    });
}

#[test]
fn only_providers_can_attest() {
    new_test_ext().execute_with(|| {
        // only "root" can register
        assert_noop!(Membership::add_member(Origin::signed(2), 2), BadOrigin);
        assert_noop!(
            TestProvider::attest(Origin::signed(2), BTC, 123),
            Error::<Test>::NotAProvider
        );
        assert_ok!(Membership::add_member(Origin::signed(ROOT), 2));
        assert_ok!(TestProvider::attest(Origin::signed(2), BTC, 123));
    });
}
