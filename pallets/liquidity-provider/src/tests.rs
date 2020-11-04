use crate::mock::*;
use alloc::vec;
use frame_support::{assert_noop, assert_ok};
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use sp_runtime::traits::BadOrigin;
use valiu_node_commons::{AccountRate, Asset, Collateral, DistributionStrategy, OfferRate};

const ROOT: u64 = 1;
const USD_ASSET: Asset = Asset::Collateral(USD_COLLATERAL);
const USD_COLLATERAL: Collateral = Collateral::Usd;
const USDC_ASSET: Asset = Asset::Collateral(USDC_COLLATERAL);
const USDC_COLLATERAL: Collateral = Collateral::Usdc;
const USDV_ASSET: Asset = Asset::Usdv;

#[test]
fn attest_increases_usdv() {
    new_test_ext().execute_with(|| {
        assert_ok!(ProviderMembers::add_member(Origin::signed(ROOT), 2));
        assert_ok!(TestProvider::attest(
            Origin::signed(2),
            USDC_ASSET,
            123,
            Default::default()
        ));
        assert_eq!(Tokens::free_balance(USDC_ASSET, &2), 0);
        assert_eq!(Tokens::reserved_balance(USDC_ASSET, &2), 123);
        assert_eq!(Tokens::free_balance(USDV_ASSET, &2), 123);
        assert_eq!(Tokens::total_issuance(USDC_ASSET), 123);
    });
}

#[test]
fn must_be_provider_to_attest() {
    new_test_ext().execute_with(|| {
        assert_noop!(ProviderMembers::add_member(Origin::signed(2), 2), BadOrigin);

        let _ = TestProvider::attest(Origin::signed(2), USDC_ASSET, 123, Default::default());

        assert_noop!(
            TestProvider::attest(Origin::signed(2), USDC_ASSET, 123, Default::default()),
            pallet_membership::Error::<Test, crate::ProviderMembers>::NotMember
        );

        assert_ok!(ProviderMembers::add_member(Origin::signed(ROOT), 2));
        assert_ok!(TestProvider::attest(
            Origin::signed(2),
            USDC_ASSET,
            123,
            Default::default()
        ));
    });
}

#[test]
fn rate_offers_are_modified_when_attesting_or_updating() {
    new_test_ext().execute_with(|| {
        assert_ok!(ProviderMembers::add_member(Origin::signed(ROOT), 2));
        let mut offers = vec![OfferRate::new(USDC_ASSET, 123)];
        assert_ok!(TestProvider::attest(
            Origin::signed(2),
            USD_ASSET,
            123,
            offers.clone()
        ));
        assert_eq!(
            TestProvider::account_rates(&USD_ASSET, &USDC_ASSET),
            vec![AccountRate::new(2, 123)]
        );
        offers[0] = OfferRate::new(USDC_ASSET, 100);
        assert_ok!(TestProvider::update_offer_rates(
            Origin::signed(2),
            USD_ASSET,
            offers
        ));
        assert_eq!(
            TestProvider::account_rates(&USD_ASSET, &USDC_ASSET),
            vec![AccountRate::new(2, 100)]
        );
    });
}

#[test]
fn usdv_transfer_also_transfers_collaterals() {
    new_test_ext().execute_with(|| {
        assert_ok!(ProviderMembers::add_member(Origin::signed(ROOT), 2));

        assert_ok!(ProviderMembers::add_member(Origin::signed(ROOT), 3));

        assert_ok!(TestProvider::attest(
            Origin::signed(2),
            USD_ASSET,
            60,
            Default::default()
        ));
        assert_ok!(TestProvider::attest(
            Origin::signed(2),
            USDC_ASSET,
            40,
            Default::default()
        ));

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