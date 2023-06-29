use crate::Kusama;
use crate::*;
use integration_tests_common::constants::XCM_V3;
use xcm_emulator::{Parachain as Para, RelayChain as Relay};
use frame_support::traits::PalletInfoAccess;
use thousands::Separable;

#[test]
fn reserve_transfer_native_token_from_relay_chain_parachain_to_kreivo_parachain() {
	// Init tests variables
	let amount = 1000000000000;
	let relay_sender_balance_before = Kusama::account_data_of(KusamaSender::get()).free;
	let para_receiver_balance_before = Kreivo::account_data_of(KreivoReceiver::get()).free;

	let origin = <Kusama as Relay>::RuntimeOrigin::signed(KusamaSender::get());
	let root_kusama = <Kusama as Relay>::RuntimeOrigin::root();
	let kreivo_para_destination: VersionedMultiLocation = Kusama::child_location_of(Kreivo::para_id()).into();
	let kreivo_remote: MultiLocation = Kusama::child_location_of(Kreivo::para_id());
	let beneficiary: VersionedMultiLocation = AccountId32 {
		network: None,
		id: KreivoReceiver::get().into(),
	}
	.into();
	let native_assets: VersionedMultiAssets = (Here, amount).into();
	let fee_asset_item = 0;
	let weight_limit = WeightLimit::Unlimited;

	// Send XCM message from Relay Chain
	Kusama::execute_with(|| {
		assert_ok!(<Kusama as KusamaPallet>::XcmPallet::force_xcm_version(
			root_kusama,
			bx!(kreivo_remote),
			XCM_V3
		));
		assert_ok!(<Kusama as KusamaPallet>::XcmPallet::limited_reserve_transfer_assets(
			origin,
			bx!(kreivo_para_destination),
			bx!(beneficiary),
			bx!(native_assets),
			fee_asset_item,
			weight_limit,
		));

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

    let mut beneficiary_balance = 0;
	// Receive XCM message in Assets Parachain
	Kreivo::execute_with(|| {
		type RuntimeEvent = <Kreivo as Para>::RuntimeEvent;

        println!("kreivo events ${:?}", Kreivo::events());

		 		assert_expected_events!(
			Kreivo,
			vec![
				RuntimeEvent::DmpQueue(cumulus_pallet_dmp_queue::Event::ExecutedDownward {
					outcome: Outcome::Complete(_),
					..
				}) => {},
			]
		);
	});
    const EST_FEES: u128 = 1_600_000_000 * 10;

	let relay_sender_balance_after = Kusama::account_data_of(KusamaSender::get()).free;
	let para_sender_balance_after = Kreivo::account_data_of(KreivoReceiver::get()).free;


    println!(
        "Reserve-transfer: initial balance {} transfer amount {} current balance {} estimated fees {} actual fees {}",
        para_receiver_balance_before.separate_with_commas(),
        amount.separate_with_commas(),
        para_sender_balance_after.separate_with_commas(),
        EST_FEES.separate_with_commas(),
        ( amount + para_receiver_balance_before - para_sender_balance_after).separate_with_commas()
    );

    assert_balance(para_sender_balance_after, para_receiver_balance_before + amount, EST_FEES);
}

#[test]
fn reserve_transfer_asset_from_statemine_parachain_to_kreivo_parachain() {
	// Init tests variables
    const ASSET_ID: u32 = 1984;
    const AMOUNT: u128 = 20_000_000_000;
    const MINT_AMOUNT: u128 = 100_000_000_000_000;
    let root_statemine = <Statemine as Para>::RuntimeOrigin::root();
    let kreivo_root = <Kreivo as Para>::RuntimeOrigin::root();

    let statemine_remote: MultiLocation = MultiLocation {
        parents: 1,
        interior: X1(Parachain(Statemine::para_id().into())),
    };
    let kreivo_remote: MultiLocation = MultiLocation {
        parents: 1,
        interior: X1(Parachain(Kreivo::para_id().into())),
    };
	let statemine_origin = <Statemine  as Para>::RuntimeOrigin::signed(StatemineSender::get());
    let beneficiary: VersionedMultiLocation = AccountId32 {
		network: None,
		id: KreivoReceiver::get().into(),
	}.into();
    let asset_to_transfer: VersionedMultiAssets = (X2(PalletInstance(50.into()), GeneralIndex(ASSET_ID as u128)), AMOUNT).into();
	let fee_asset_item = 0;
	let weight_limit = WeightLimit::Unlimited;

    Kreivo::execute_with(|| {
		type RuntimeEvent = <Kreivo as Para>::RuntimeEvent;

        assert_ok!(<Kreivo as KreivoPallet>::PolkadotXcm::force_xcm_version(
            kreivo_root.clone(),
			bx!(statemine_remote),
			XCM_V3
		));

        assert_ok!(<Kreivo as KreivoPallet>::Assets::force_create(
            kreivo_root.clone(),
            ASSET_ID.into(),
            KreivoSender::get().into(),
            true,
            10000u128.into(),
		));

        assert_ok!(<Kreivo as KreivoPallet>::AssetRegistry::register_reserve_asset(
            kreivo_root.clone(),
            ASSET_ID.into(),
            (
                Parent,
                X3(
                    Parachain(Statemine::para_id().into()),
                    PalletInstance(<Statemine as StateminePallet>::Assets::index() as u8),
                    GeneralIndex(ASSET_ID.into()),
                ),
            )
                .into(),
		));
	});

    Statemine::execute_with(|| {
        assert_ok!(<Statemine as StateminePallet>::Assets::force_create(
            root_statemine.clone(),
            ASSET_ID.into(),
            StatemineSender::get().into(),
            true,
            10000u128.into(),
		));

        assert_ok!(<Statemine as StateminePallet>::Assets::mint(
            <Statemine as Para>::RuntimeOrigin::signed(StatemineSender::get()),
            ASSET_ID.into(),
            StatemineSender::get().into(),
            MINT_AMOUNT.into(),
		));

        assert_ok!(<Statemine as StateminePallet>::PolkadotXcm::force_xcm_version(
            root_statemine,
			bx!(kreivo_remote),
			XCM_V3
		));

        assert_ok!(<Statemine as StateminePallet>::PolkadotXcm::limited_reserve_transfer_assets(
            <Statemine as Para>::RuntimeOrigin::signed(StatemineSender::get()),
			bx!(kreivo_remote.into()),
			bx!(beneficiary),
			bx!(asset_to_transfer),
			fee_asset_item,
			weight_limit,
		));
    });

    Kreivo::execute_with(|| {
		type RuntimeEvent = <Kreivo as Para>::RuntimeEvent;

        let balance = <Kreivo as KreivoPallet>::Assets::balance(ASSET_ID.into(), KreivoReceiver::get());
	});
}

