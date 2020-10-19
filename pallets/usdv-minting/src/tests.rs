use crate::mock::*;
use frame_support::assert_ok;
use valiu_node_commons::ValiuCurrencies;

const USDV: ValiuCurrencies = ValiuCurrencies::Usdv;
const ROOT: u64 = 1;

#[test]
fn mint_increases_usdv_supply() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::add_member(Origin::signed(ROOT), 2));
        assert_ok!(Membership::add_member(Origin::signed(ROOT), 3));

        assert_ok!(TestProvider::mint(Origin::signed(2), 123));
        assert_ok!(TestProvider::mint(Origin::signed(3), 456));
        assert_eq!(Tokens::total_issuance(USDV), 579);
    });
}