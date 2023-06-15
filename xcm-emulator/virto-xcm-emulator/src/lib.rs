use codec::Compact;
use frame_support::{pallet_prelude::Weight, traits::GenesisBuild};
use polkadot_primitives::runtime_api::runtime_decl_for_parachain_host::ParachainHostV4;
mod runtimes;
use runtimes::*;
use std::sync::Once;
use thousands::Separable;
use xcm::prelude::*;
use xcm_emulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};

const ASSET_RESERVE_PARA_ID: u32 = runtimes::asset_reserve::ASSET_RESERVE_PARA_ID;
const KREIVO_PARA_ID: u32 = runtimes::kreivo::KREIVO_PARA_ID;

#[allow(non_upper_case_globals)]
const xUSD: u32 = 1984;
#[allow(non_upper_case_globals)]
const txUSD: u32 = 10;

decl_test_relay_chain! {
	pub struct RococoNet {
		Runtime = rococo_runtime::Runtime,
		XcmConfig = rococo_runtime::xcm_config::XcmConfig,
		new_ext = runtimes::relay_chain::new_ext(),
	}
}

decl_test_parachain! {
	pub struct AssetReserveParachain {
		Runtime = statemine_runtime::Runtime,
		RuntimeOrigin = statemine_runtime::RuntimeOrigin,
		XcmpMessageHandler = statemine_runtime::XcmpQueue,
		DmpMessageHandler = statemine_runtime::DmpQueue,
		new_ext = runtimes::asset_reserve::new_ext(ASSET_RESERVE_PARA_ID),
	}
}

decl_test_parachain! {
	pub struct KreivoParachain {
		Runtime = kreivo_runtime::Runtime,
		RuntimeOrigin = kreivo_runtime::RuntimeOrigin,
		XcmpMessageHandler = kreivo_runtime::XcmpQueue,
		DmpMessageHandler = kreivo_runtime::DmpQueue,
		new_ext = runtimes::kreivo::new_ext(KREIVO_PARA_ID),
	}
}

decl_test_network! {
	pub struct Network {
		relay_chain = RococoNet,
		parachains = vec![
			(1_000, AssetReserveParachain),
			(2_000, KreivoParachain),
		],
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use codec::Encode;

	use cumulus_primitives_core::ParaId;
	use frame_support::{assert_ok, dispatch::GetDispatchInfo, traits::Currency};
	use sp_runtime::{traits::AccountIdConversion, MultiAddress};
	use xcm::{v3::prelude::*, VersionedMultiLocation, VersionedXcm};
	use xcm_emulator::TestExt;

	#[test]
	fn reserve_transfer_asset_from_asset_reserve_parachain_to_kreivo_parachain() {
		init_tracing();

		Network::reset();

		const ASSET_MIN_BALANCE: u128 = 1_000_000_000;
		const MINT_AMOUNT: u128 = 1_000_000_000_000_000_000;

		let kreivo_remote: MultiLocation = MultiLocation {
			parents: 1,
			interior: X1(Parachain(KREIVO_PARA_ID)),
		};

		let parent: MultiLocation = MultiLocation {
			parents: 1,
			interior: Here,
		};

		AssetReserveParachain::execute_with(|| {
			println!("     ");
			println!(
				">>>>>>>>> AssetReserveParachain: set XCM versions and set sufficient assets <<<<<<<<<<<<<<<<<<<<<<"
			);
			println!("     ");
			assert_ok!(statemine_runtime::PolkadotXcm::force_xcm_version(
				statemine_runtime::RuntimeOrigin::root(),
				Box::new(parent.clone().into()),
				XCM_VERSION
			));

			assert_ok!(statemine_runtime::PolkadotXcm::force_xcm_version(
				statemine_runtime::RuntimeOrigin::root(),
				Box::new(kreivo_remote.clone()),
				XCM_VERSION
			));

			assert_ok!(create_asset_on_asset_reserve(xUSD, ALICE, 1_000_000_000));

			// Mint fungible asset
			assert_ok!(mint_asset_on_asset_reserve(xUSD, ALICE, MINT_AMOUNT));
			assert_eq!(statemine_runtime::Assets::balance(xUSD, &ALICE), MINT_AMOUNT);

			assert_ok!(statemine_runtime::Assets::force_asset_status(
				statemine_runtime::RuntimeOrigin::root(),
				Compact(xUSD),
				ALICE.into(),
				ALICE.into(),
				ALICE.into(),
				ALICE.into(),
				ASSET_MIN_BALANCE,
				true,
				false,
			));

			statemine_runtime::System::events()
				.iter()
				.for_each(|r| println!(">>> {:?}", r.event));

			statemine_runtime::System::assert_has_event(
				pallet_xcm::Event::SupportedVersionChanged(
					MultiLocation {
						parents: 1,
						interior: Here,
					},
					3,
				)
				.into(),
			);

			statemine_runtime::System::assert_has_event(
				pallet_xcm::Event::SupportedVersionChanged(kreivo_remote.clone(), 3).into(),
			);
		});

		RococoNet::execute_with(|| {
			println!("     ");
			println!(">>>>>>>>> RococoNet: force xcm v3 version <<<<<<<<<<<<<<<<<<<<<<");
			println!("     ");
			assert_ok!(rococo_runtime::XcmPallet::force_default_xcm_version(
				rococo_runtime::RuntimeOrigin::root(),
				Some(XCM_VERSION)
			));
		});

		let mut beneficiary_balance = 0;
		// 7) Create derivative asset on Trappist Parachain
		// 8) Sets the asset as sufficient on Trappist	Parachain
		KreivoParachain::execute_with(|| {
			println!("     ");
			println!(">>>>>>>>> KreivoParachain: create derivative asset and fund statemine sov account <<<<<<<<<<<<<<<<<<<<<<");
			println!("     ");
			let statemine_sovereign_account = runtimes::sovereign_account(ASSET_RESERVE_PARA_ID);

			assert_ok!(kreivo_runtime::Balances::transfer(
				kreivo_runtime::RuntimeOrigin::signed(ALICE),
				MultiAddress::Id(statemine_sovereign_account.clone()),
				1_000_000_000_000
			));

			// Create derivative asset on Trappist Parachain
			assert_ok!(create_derivative_asset_on_kreivo(
				txUSD,
				ALICE.into(),
				ASSET_MIN_BALANCE
			));

			assert_ok!(kreivo_runtime::Assets::force_asset_status(
				kreivo_runtime::RuntimeOrigin::root(),
				Compact(txUSD),
				ALICE.into(),
				ALICE.into(),
				ALICE.into(),
				ALICE.into(),
				ASSET_MIN_BALANCE,
				true,
				false,
			));

			// Map derivative asset (txUSD) to multi-location (xUSD within Assets pallet on
			// Reserve Parachain) via Asset Registry
			assert_ok!(register_reserve_asset_on_kreivo(ALICE, txUSD, xUSD));
			kreivo_runtime::System::assert_has_event(
				pallet_asset_registry::Event::ReserveAssetRegistered {
					asset_id: txUSD,
					asset_multi_location: MultiLocation {
						parents: 1,
						interior: Junctions::X3(
							Parachain(ASSET_RESERVE_PARA_ID),
							PalletInstance(50),
							GeneralIndex(xUSD.into()),
						),
					},
				}
				.into(),
			);
			kreivo_runtime::System::events()
				.iter()
				.for_each(|r| println!(">>> {:?}", r.event));
		});

		const AMOUNT: u128 = 20_000_000_000;
		// 8) Fund Trappist sovereign account on Reserve Parachain
		// 9) Sends XCM to Trappist Parachain to reserve-transfer an asset to Trappist
		// Parachain
		AssetReserveParachain::execute_with(|| {
			println!("     ");
			println!(">>>>>>>>> AssetReserveParachain: reserve based transfer<<<<<<<<<<<<<<<<<<<<<<");
			println!("     ");

			let kreivo_sovereign_account = runtimes::sovereign_account(KREIVO_PARA_ID);

			assert_ok!(statemine_runtime::Balances::transfer(
				statemine_runtime::RuntimeOrigin::signed(ALICE),
				MultiAddress::Id(kreivo_sovereign_account.clone()),
				1_000_000_000_000
			));

			// Reserve parachain should be able to reserve-transfer an asset to Trappist
			// Parachain
			assert_ok!(statemine_runtime::PolkadotXcm::limited_reserve_transfer_assets(
				statemine_runtime::RuntimeOrigin::signed(ALICE),
				Box::new(kreivo_remote.clone().into()),
				Box::new(
					X1(AccountId32 {
						network: Some(NetworkId::Rococo),
						id: ALICE.into()
					})
					.into()
				),
				Box::new((X2(PalletInstance(50.into()), GeneralIndex(xUSD as u128)), AMOUNT).into()),
				0,
				WeightLimit::Unlimited,
			));

			statemine_runtime::System::events()
				.iter()
				.for_each(|r| println!(">>> {:?}", r.event));

			assert_eq!(
				statemine_runtime::Assets::balance(xUSD, &kreivo_sovereign_account),
				AMOUNT
			);
		});

		// 10) Checks on Trappist Parachain that the asset was received
		const EST_FEES: u128 = 1_600_000_000 * 10;
		KreivoParachain::execute_with(|| {
			println!("     ");
			println!(">>>>>>>>> KreivoParachain <<<<<<<<<<<<<<<<<<<<<<");
			println!("     ");
			// Ensure beneficiary account balance increased
			let current_balance = kreivo_runtime::Assets::balance(txUSD, &ALICE);
			kreivo_runtime::System::events()
				.iter()
				.for_each(|r| println!(">>> {:?}", r.event));

			println!(
				"Reserve-transfer: initial balance {} transfer amount {} current balance {} estimated fees {} actual fees {}",
				beneficiary_balance.separate_with_commas(),
				AMOUNT.separate_with_commas(),
				current_balance.separate_with_commas(),
				EST_FEES.separate_with_commas(),
				(beneficiary_balance + AMOUNT - current_balance).separate_with_commas()
			);
			runtimes::assert_balance(current_balance, beneficiary_balance + AMOUNT, EST_FEES);
		});
	}
}
