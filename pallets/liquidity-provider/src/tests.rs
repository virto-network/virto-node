use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use sp_runtime::traits::BadOrigin;
use valiu_node_commons::{Asset, Collateral};

const ROOT: u64 = 1;
const USDC_ASSET: Asset = Asset::Collateral(USDC_COLLATERAL);
const USDC_COLLATERAL: Collateral = Collateral::Usdc;
const USDV_ASSET: Asset = Asset::Usdv;

#[test]
fn attest_increases_supply() {
    new_test_ext().execute_with(|| {
        assert_ok!(MintMembers::add_member(Origin::signed(ROOT), 2));
        assert_ok!(ProviderMembers::add_member(Origin::signed(ROOT), 2));
        assert_ok!(TestProvider::attest(
            Origin::signed(2),
            USDC_COLLATERAL,
            123
        ));
        assert_eq!(Tokens::free_balance(USDC_ASSET, &2), 0);
        assert_eq!(Tokens::reserved_balance(USDC_ASSET, &2), 123);
        assert_eq!(Tokens::free_balance(USDV_ASSET, &2), 123);

        assert_ok!(MintMembers::add_member(Origin::signed(ROOT), 3));
        assert_ok!(ProviderMembers::add_member(Origin::signed(ROOT), 3));
        assert_ok!(TestProvider::attest(
            Origin::signed(3),
            USDC_COLLATERAL,
            456
        ));
        assert_eq!(Tokens::free_balance(USDC_ASSET, &3), 0);
        assert_eq!(Tokens::reserved_balance(USDC_ASSET, &3), 456);
        assert_eq!(Tokens::free_balance(USDV_ASSET, &3), 456);

        assert_eq!(Tokens::total_issuance(USDC_ASSET), 579);
    });
}

#[test]
fn only_providers_can_attest() {
    new_test_ext().execute_with(|| {
        assert_noop!(MintMembers::add_member(Origin::signed(2), 2), BadOrigin);
        assert_noop!(ProviderMembers::add_member(Origin::signed(2), 2), BadOrigin);

        assert_noop!(
            TestProvider::attest(Origin::signed(2), USDC_COLLATERAL, 123),
            pallet_membership::Error::<Test, pallet_membership::DefaultInstance>::NotMember
        );
        assert_noop!(
            TestProvider::attest(Origin::signed(2), USDC_COLLATERAL, 123),
            pallet_membership::Error::<Test, pallet_membership::Instance0>::NotMember
        );

        assert_ok!(MintMembers::add_member(Origin::signed(ROOT), 2));
        assert_ok!(ProviderMembers::add_member(Origin::signed(ROOT), 2));
        assert_ok!(TestProvider::attest(
            Origin::signed(2),
            USDC_COLLATERAL,
            123
        ));
    });
}
