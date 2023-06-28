use crate::{Kusama, init_tracing};
use crate::*;
use xcm_emulator::{Parachain as Para, RelayChain as Relay};

#[test]
fn reserve_transfer_native_asset_from_relay_to_assets() {

    init_tracing();

    // Init tests variables
    let amount = KUSAMA_ED * 1000;
    let relay_sender_balance_before = Kusama::account_data_of(KusamaSender::get()).free;
    let para_receiver_balance_before = Statemine::account_data_of(StatemineReceiver::get()).free;

    let origin = <Kusama as Relay>::RuntimeOrigin::signed(KusamaSender::get());
    let assets_para_destination: VersionedMultiLocation =
        Kusama::child_location_of(Statemine::para_id()).into();
    let beneficiary: VersionedMultiLocation = AccountId32 {
        network: None,
        id: StatemineReceiver::get().into(),
    }
    .into();
    let native_assets: VersionedMultiAssets = (Here, amount).into();
    let fee_asset_item = 0;
    let weight_limit = WeightLimit::Unlimited;

    // Send XCM message from Relay Chain
    Kusama::execute_with(|| {
        assert_ok!(
            <Kusama as KusamaPallet>::XcmPallet::limited_reserve_transfer_assets(
                origin,
                bx!(assets_para_destination),
                bx!(beneficiary),
                bx!(native_assets),
                fee_asset_item,
                weight_limit,
            )
        );

        type RuntimeEvent = <Kusama as Relay>::RuntimeEvent;

        assert_expected_events!(
            Kusama,
            vec![
                RuntimeEvent::XcmPallet(pallet_xcm::Event::Attempted(Outcome::Complete(weight))) => {
                    weight: weight_within_threshold((REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD), Weight::from_parts(754_244_000, 0), *weight),
                },
            ]
        );
    });

    // Receive XCM message in Assets Parachain
    Statemine::execute_with(|| {
        type RuntimeEvent = <Statemine as Para>::RuntimeEvent;

        assert_expected_events!(
            Statemine,
            vec![
                RuntimeEvent::DmpQueue(cumulus_pallet_dmp_queue::Event::ExecutedDownward {
                    outcome: Outcome::Incomplete(_, Error::UntrustedReserveLocation),
                    ..
                }) => {},
            ]
        );
    });

    // Check if balances are updated accordingly in Relay Chain and Assets Parachain
    let relay_sender_balance_after = Kusama::account_data_of(KusamaSender::get()).free;
    let para_sender_balance_after = Statemine::account_data_of(StatemineReceiver::get()).free;

    assert_eq!(
        relay_sender_balance_before - amount,
        relay_sender_balance_after
    );
    assert_eq!(para_sender_balance_after, para_receiver_balance_before);
}

#[test]
fn reserve_transfer_asset_from_relay_chain_parachain_to_kreivo_parachain() {
    init_tracing();

    let kreivo_location: MultiLocation = MultiLocation {
        parents: 0,
        interior: X1(Parachain(KREIVO_PARA_ID)),
    };

    const AMOUNT: u128 = 5_000_000_000_000;

    Kusama::execute_with(|| {
        println!("     ");
        println!(">>>>>>>>> Kusama: force xcm v3 version <<<<<<<<<<<<<<<<<<<<<<");
        println!("     ");
        assert_ok!(kusama_runtime::XcmPallet::force_default_xcm_version(
            kusama_runtime::RuntimeOrigin::root(),
            Some(XCM_VERSION)
        ));

        assert_ok!(kusama_runtime::XcmPallet::limited_reserve_transfer_assets(
            kusama_runtime::RuntimeOrigin::signed(ALICE),
            Box::new(kreivo_location.clone().into()),
            Box::new(
                X1(AccountId32 {
                    network: None,
                    id: ALICE.into()
                })
                .into()
            ),
            Box::new((Here, AMOUNT).into()),
            0,
            WeightLimit::Unlimited,
        ));
    });

    Kreivo::execute_with(|| {
        println!("     ");
        println!(">>>>>>>>> KreivoParachain <<<<<<<<<<<<<<<<<<<<<<");
        println!("     ");

        println!(
            "ALICE Balance on Kreivo: {:?}",
            kreivo_runtime::Balances::free_balance(&ALICE)
        );

        // Ensure beneficiary account balance increased
        kreivo_runtime::System::events()
            .iter()
            .for_each(|r| println!(">>> {:?}", r.event));

        // TODO: check that the balance is increased
    });
}