use crate::{
    mock::{
        new_test_ext, Extrinsic, Origin, ProviderMembers, Test, TestAuth, TestProvider, Tokens,
    },
    Call, OffchainPairPricesPayload,
};
use alloc::vec;
use frame_support::{assert_noop, assert_ok};
use frame_system::offchain::{SignedPayload, SigningTypes};
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use parity_scale_codec::Decode;
use sp_core::{
    offchain::{testing, OffchainExt, TransactionPoolExt},
    sr25519,
    testing::KeyStore,
    traits::KeystoreExt,
};
use sp_io::TestExternalities;
use sp_runtime::{traits::BadOrigin, RuntimeAppPublic};
use valiu_node_commons::{
    AccountRate, Asset, Collateral, DistributionStrategy, OfferRate, PairPrice,
};

const USD_ASSET: Asset = Asset::Collateral(USD_COLLATERAL);
const USD_COLLATERAL: Collateral = Collateral::Usd;
const USDC_ASSET: Asset = Asset::Collateral(USDC_COLLATERAL);
const USDC_COLLATERAL: Collateral = Collateral::Usdc;
const USDV_ASSET: Asset = Asset::Usdv;

fn alice() -> sr25519::Public {
    <sr25519::Public>::from_raw({
        let mut array = [0; 32];
        array[31] = 2;
        array
    })
}
fn root() -> sr25519::Public {
    <sr25519::Public>::from_raw([0; 32])
}
fn bob() -> sr25519::Public {
    <sr25519::Public>::from_raw({
        let mut array = [0; 32];
        array[31] = 1;
        array
    })
}

#[test]
fn attest_increases_usdv() {
    new_test_ext().execute_with(|| {
        let alice = alice();

        assert_ok!(ProviderMembers::add_member(Origin::signed(root()), alice));
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
            pallet_membership::Error::<Test, crate::ProviderMembers>::NotMember
        );

        assert_ok!(ProviderMembers::add_member(Origin::signed(root()), alice));
        assert_ok!(TestProvider::attest(
            Origin::signed(alice),
            USDC_ASSET,
            123,
            Default::default()
        ));
    });
}

#[test]
fn offchain_worker_submits_unsigned_transaction_on_chain() {
    new_test_ext().execute_with(|| {
        const SEED: Option<&str> =
            Some("news slush supreme milk chapter athlete soap sausage put clutch what kitten/foo");

        let (offchain, offchain_state) = testing::TestOffchainExt::new();

        let (pool, pool_state) = testing::TestTransactionPoolExt::new();

        let keystore = KeyStore::new();

        let public_key = keystore
            .write()
            .sr25519_generate_new(crate::Public::ID, SEED)
            .unwrap();

        let mut t = TestExternalities::default();
        t.register_extension(OffchainExt::new(offchain));
        t.register_extension(TransactionPoolExt::new(pool));
        t.register_extension(KeystoreExt(keystore));

        offchain_state
            .write()
            .expect_request(testing::PendingRequest {
                method: "GET".into(),
                uri:
                    "https://min-api.cryptocompare.com/data/pricemulti?fsyms=BTC,USD&tsyms=BTC,USD"
                        .into(),
                response: Some(br#"{"BTC":{"BTC":1,"USD":200},"USD":{"BTC":2,"USD":1}}"#.to_vec()),
                sent: true,
                ..Default::default()
            });

        offchain_state
            .write()
            .expect_request(testing::PendingRequest {
                method: "GET".into(),
                uri: "https://www.trmhoy.co/".into(),
                response: Some(
                    br#"<div id="banner">Te Compran <h3>$ 120</h3> Te Venden <h3>$ 12</h3></div>"#
                        .to_vec(),
                ),
                sent: true,
                ..Default::default()
            });

        let payload = OffchainPairPricesPayload {
            pair_prices: vec![
                PairPrice::new([Asset::Btc, Asset::Collateral(Collateral::Usd)], 200, 2),
                PairPrice::new([Asset::Collateral(Collateral::Usd), Asset::Cop], 120, 12),
            ],
            public: <Test as SigningTypes>::Public::from(public_key),
        };

        t.execute_with(|| {
            // when
            TestProvider::fetch_pair_prices_and_submit_tx(1).unwrap();

            // then
            let raw_tx = pool_state.write().transactions.pop().unwrap();
            let tx = Extrinsic::decode(&mut &*raw_tx).unwrap();
            assert_eq!(tx.signature, None);
            if let Call::submit_pair_prices(body, signature) = tx.call {
                assert_eq!(body, payload.pair_prices);
                let signature_valid = <OffchainPairPricesPayload<
                    <Test as frame_system::Trait>::BlockNumber,
                    <Test as SigningTypes>::Public,
                > as SignedPayload<Test>>::verify::<TestAuth>(
                    &payload, signature
                );
                assert!(signature_valid);
            }
        });
    })
}

#[test]
fn rate_offers_are_modified_when_attesting_or_updating() {
    new_test_ext().execute_with(|| {
        let alice = alice();

        assert_ok!(ProviderMembers::add_member(Origin::signed(root()), alice));

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

#[test]
fn usdv_transfer_also_transfers_collaterals() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        let bob = bob();

        assert_ok!(ProviderMembers::add_member(Origin::signed(root()), alice));

        assert_ok!(ProviderMembers::add_member(Origin::signed(root()), bob));

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
            bob,
            30,
            DistributionStrategy::Evenly
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
fn parse_btc_usd_has_correct_behavior() {
    assert_eq!(
        TestProvider::parse_btc_usd(r#"{"BTC":{"BTC":1,"USD":200},"USD":{"BTC":2,"USD":1}}"#),
        Some(PairPrice::new(
            [Asset::Btc, Asset::Collateral(Collateral::Usd)],
            200,
            2,
        ))
    );
    assert!(TestProvider::parse_btc_usd(
        r#"{"BTC":{"BTC":1,"USD":"foo"},"USD":{"BTC":2,"USD":1}}"#
    )
    .is_none());
    assert!(
        TestProvider::parse_btc_usd(r#"{"btc":{"btc":1,"usd":200},"usd":{"btc":2,"usd":1}}"#)
            .is_none()
    );
}
