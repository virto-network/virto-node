use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use sp_runtime::traits::BadOrigin;
use valiu_node_commons::{Asset, Collateral, DistributionStrategy};

const ROOT: u64 = 1;
const USD_ASSET: Asset = Asset::Collateral(USD_COLLATERAL);
const USD_COLLATERAL: Collateral = Collateral::Usd;
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

#[test]
fn usdv_transfer_also_transfers_collaterals() {
    new_test_ext().execute_with(|| {
        assert_ok!(MintMembers::add_member(Origin::signed(ROOT), 2));
        assert_ok!(ProviderMembers::add_member(Origin::signed(ROOT), 2));

        assert_ok!(ProviderMembers::add_member(Origin::signed(ROOT), 3));

        assert_ok!(TestProvider::attest(Origin::signed(2), USD_COLLATERAL, 60));
        assert_ok!(TestProvider::attest(Origin::signed(2), USDC_COLLATERAL, 40));

        assert_ok!(TestProvider::transfer(
            Origin::signed(2),
            3,
            30,
            DistributionStrategy::Evenly
        ));

        assert_eq!(Tokens::free_balance(USDV_ASSET, &2), 70);
        assert_eq!(Tokens::free_balance(USDV_ASSET, &3), 30);

        assert_eq!(Tokens::reserved_balance(USD_ASSET, &2), 42);
        assert_eq!(Tokens::reserved_balance(USDC_ASSET, &2), 28);
        assert_eq!(Tokens::reserved_balance(USD_ASSET, &3), 18);
        assert_eq!(Tokens::reserved_balance(USDC_ASSET, &3), 12);

        assert_eq!(Tokens::total_issuance(USD_ASSET), 60);
        assert_eq!(Tokens::total_issuance(USDC_ASSET), 40);
        assert_eq!(Tokens::total_issuance(USDV_ASSET), 100);
    });
}
