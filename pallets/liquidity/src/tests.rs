use crate::mock::{AccountId, Origin, ProviderMembers, Test, TestProvider, Tokens, USD_ASSET};
use frame_support::{assert_noop, assert_ok};
use once_cell::sync::Lazy;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use parking_lot::RwLock;
use sp_core::{
    testing::KeyStore,
    traits::{BareCryptoStore, KeystoreExt},
    H256,
};
use sp_io::TestExternalities;
use sp_runtime::traits::BadOrigin;
use std::sync::Arc;
use vln_commons::{AccountRate, Asset, Collateral, Destination, OfferRate};

const USDC_ASSET: Asset = Asset::Collateral(USDC_COLLATERAL);
const USDC_COLLATERAL: Collateral = Collateral::Usdc;
const USDV_ASSET: Asset = Asset::Usdv;

static KEYSTORE: Lazy<Arc<RwLock<dyn BareCryptoStore>>> = Lazy::new(|| KeyStore::new());

#[test]
fn attest_increases_usdv() {
    new_test_ext().execute_with(|| {
        let alice = alice();

        assert_ok!(ProviderMembers::add_member(Origin::root(), alice));
        assert_ok!(TestProvider::attest(
            Origin::signed(alice),
            USDC_ASSET,
            123,
            Default::default()
        ));
        assert_eq!(Tokens::free_balance(USDC_ASSET, &alice), 0);
        assert_eq!(Tokens::reserved_balance(USDC_ASSET, &alice), 123);
        assert_eq!(Tokens::free_balance(USDV_ASSET, &alice), 123);
        assert_eq!(Tokens::total_issuance(USDC_ASSET), 123);
    });
}

#[test]
fn must_be_provider_to_attest() {
    new_test_ext().execute_with(|| {
        let alice = alice();

        assert_noop!(
            ProviderMembers::add_member(Origin::signed(alice), alice),
            BadOrigin
        );

        let _ = TestProvider::attest(Origin::signed(alice), USDC_ASSET, 123, Default::default());

        assert_noop!(
            TestProvider::attest(Origin::signed(alice), USDC_ASSET, 123, Default::default()),
            pallet_membership::Error::<Test, crate::LiquidityMembers>::NotMember
        );

        assert_ok!(ProviderMembers::add_member(Origin::root(), alice));
        assert_ok!(TestProvider::attest(
            Origin::signed(alice),
            USDC_ASSET,
            123,
            Default::default()
        ));
    });
}

#[test]
fn rate_offers_are_modified_when_attesting_or_updating() {
    new_test_ext().execute_with(|| {
        let alice = alice();

        assert_ok!(ProviderMembers::add_member(Origin::root(), alice));

        let mut offers = vec![OfferRate::new(USDC_ASSET, 123)];

        assert_ok!(TestProvider::attest(
            Origin::signed(alice),
            USD_ASSET,
            123,
            offers.clone()
        ));

        assert_eq!(
            TestProvider::account_rates(&USD_ASSET, &USDC_ASSET),
            vec![AccountRate::new(alice, 123)]
        );

        offers[0] = OfferRate::new(USDC_ASSET, 100);

        assert_ok!(TestProvider::update_offer_rates(
            Origin::signed(alice),
            USD_ASSET,
            offers
        ));

        assert_eq!(
            TestProvider::account_rates(&USD_ASSET, &USDC_ASSET),
            vec![AccountRate::new(alice, 100)]
        );
    });
}

//#[test]
//fn update_offer_rates_overwrites_prices() {
//    new_test_ext().execute_with(|| {
//        assert_eq!(<PairPrices<Test>>::get(), vec![]);
//
//        let key = KEYSTORE
//            .write()
//            .sr25519_generate_new(crate::Public::ID, None)
//            .unwrap()
//            .into();
//
//        let first = vec![
//            PairPrice::new([Asset::Btc, Asset::Collateral(Collateral::Usd)], 1, 2),
//            PairPrice::new([Asset::Btc, Asset::Ves], 3, 4),
//            PairPrice::new([Asset::Collateral(Collateral::Usd), Asset::Cop], 5, 6),
//        ];
//        let first_sig = Signature::from_slice(
//            &KEYSTORE
//                .read()
//                .sign_with(OFFCHAIN_KEY_TYPE, &key, &first.encode())
//                .unwrap(),
//        );
//        assert_ok!(TestProvider::submit_pair_prices(
//            Origin::none(),
//            first.clone(),
//            first_sig
//        ));
//        assert_eq!(<PairPrices<Test>>::get(), first);
//
//        let second = vec![
//            PairPrice::new([Asset::Btc, Asset::Collateral(Collateral::Usd)], 7, 8),
//            PairPrice::new([Asset::Btc, Asset::Ves], 9, 10),
//            PairPrice::new([Asset::Collateral(Collateral::Usd), Asset::Cop], 11, 12),
//        ];
//        let second_sig = Signature::from_slice(
//            &KEYSTORE
//                .read()
//                .sign_with(OFFCHAIN_KEY_TYPE, &key, &second.encode())
//                .unwrap(),
//        );
//        assert_ok!(TestProvider::submit_pair_prices(
//            Origin::none(),
//            second.clone(),
//            second_sig
//        ));
//        assert_eq!(<PairPrices<Test>>::get(), second);
//    });
//}

#[test]
fn usdv_transfer_also_transfers_collaterals() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        let bob = bob();

        assert_ok!(ProviderMembers::add_member(Origin::root(), alice));
        assert_ok!(ProviderMembers::add_member(Origin::root(), bob));

        assert_ok!(TestProvider::attest(
            Origin::signed(alice),
            USD_ASSET,
            60,
            Default::default()
        ));
        assert_ok!(TestProvider::attest(
            Origin::signed(alice),
            USDC_ASSET,
            40,
            Default::default()
        ));

        assert_ok!(TestProvider::transfer(
            Origin::signed(alice),
            Destination::Vln(bob),
            30
        ));

        assert_eq!(Tokens::free_balance(USDV_ASSET, &alice), 70);
        assert_eq!(Tokens::free_balance(USDV_ASSET, &bob), 30);

        assert_eq!(Tokens::reserved_balance(USD_ASSET, &alice), 42);
        assert_eq!(Tokens::reserved_balance(USDC_ASSET, &alice), 28);
        assert_eq!(Tokens::reserved_balance(USD_ASSET, &bob), 18);
        assert_eq!(Tokens::reserved_balance(USDC_ASSET, &bob), 12);

        assert_eq!(Tokens::total_issuance(USD_ASSET), 60);
        assert_eq!(Tokens::total_issuance(USDC_ASSET), 40);
        assert_eq!(Tokens::total_issuance(USDV_ASSET), 100);
    });
}

#[test]
fn transfer_must_be_greater_than_zero() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            TestProvider::transfer(Origin::signed(alice()), Destination::Vln(bob()), 0),
            crate::Error::<Test>::TransferMustBeGreaterThanZero
        );
    });
}

#[test]
fn transfer_destinations_work_as_expected() {
    new_test_ext().execute_with(|| {
        // transfer to vln address should work - this has been tested above - skip here

        // transfer to bank destination should reject with error
        let bank_dest = Destination::Bank(H256(Default::default()));
        assert_noop!(
            TestProvider::transfer(Origin::signed(alice()), bank_dest, 0),
            crate::Error::<Test>::DestinationNotSupported
        );
    });
}

pub fn new_test_ext() -> TestExternalities {
    let mut t: TestExternalities = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into();
    t.register_extension(KeystoreExt(Arc::clone(&KEYSTORE)));
    t
}

fn alice() -> AccountId {
    <AccountId>::from_raw({
        let mut array = [0; 32];
        array[31] = 2;
        array
    })
}

fn bob() -> AccountId {
    <AccountId>::from_raw({
        let mut array = [0; 32];
        array[31] = 1;
        array
    })
}
