use frame_support::{pallet_prelude::Weight, traits::GenesisBuild};
mod runtimes;
use runtimes::*;
use sp_runtime::AccountId32;

use xcm_emulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};

const ASSET_RESERVE_PARA_ID: u32 = runtimes::asset_reserve::ASSET_RESERVE_PARA_ID;
const KREIVO_PARA_ID: u32 = runtimes::kreivo::KREIVO_PARA_ID;

const XCM_VERSION: u32 = 3;

decl_test_relay_chain! {
	pub struct KusamaNet {
		Runtime = kusama_runtime::Runtime,
		XcmConfig = kusama_runtime::xcm_config::XcmConfig,
		new_ext = runtimes::relay_chain::new_ext(),
	}
}

decl_test_parachain! {
	pub struct AssetsReserveRuntime {
		Runtime = statemine_runtime::Runtime,
		RuntimeOrigin = statemine_runtime::RuntimeOrigin,
		XcmpMessageHandler = statemine_runtime::XcmpQueue,
		DmpMessageHandler = statemine_runtime::DmpQueue,
		new_ext = runtimes::asset_reserve::new_ext(ASSET_RESERVE_PARA_ID),
	}
}

decl_test_parachain! {
	pub struct KreivoRuntime {
		Runtime = kreivo_runtime::Runtime,
		RuntimeOrigin = kreivo_runtime::RuntimeOrigin,
		XcmpMessageHandler = kreivo_runtime::XcmpQueue,
		DmpMessageHandler = kreivo_runtime::DmpQueue,
		new_ext = runtimes::kreivo::new_ext(KREIVO_PARA_ID),
	}
}

decl_test_network! {
	pub struct Network {
		relay_chain = KusamaNet,
		parachains = vec![
			(1_000, AssetsReserveRuntime),
			(2_000, KreivoRuntime),
		],
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use codec::Encode;

	use cumulus_primitives_core::ParaId;
	use frame_support::{assert_ok, dispatch::GetDispatchInfo, traits::Currency};
	use sp_runtime::traits::AccountIdConversion;
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

		// 1) Forces XCM default version on KusamaNet
		// 2) Sends Transact as Superuser to Asset Reserve Parachain to force XCM
		// default version on Asset Reserve Parachain
		KusamaNet::execute_with(|| {
			assert_ok!(kusama_runtime::XcmPallet::force_default_xcm_version(
				kusama_runtime::RuntimeOrigin::root(),
				Some(XCM_VERSION)
			));

			let force_default_xcm_version_call = statemine_runtime::RuntimeCall::PolkadotXcm(pallet_xcm::Call::<
				statemine_runtime::Runtime,
			>::force_xcm_version {
				location: Box::new(kreivo_remote.clone()),
				xcm_version: XCM_VERSION,
			});

			assert_ok!(kusama_runtime::XcmPallet::send_xcm(
				Here,
				Parachain(ASSET_RESERVE_PARA_ID),
				Xcm(vec![Transact {
					origin_type: OriginKind::Superuser,
					require_weight_at_most: 1_000_000_000,
					call: force_default_xcm_version_call.encode().into(),
				}]),
			));
		});

		// 3) Create asset xUSD
		// 4) Mint asset xUSD
		AssetReserveParachain::execute_with(|| {
			// Create fungible asset on Reserve Parachain
			assert_ok!(create_asset_on_asset_reserve(xUSD, ALICE, 1_000_000_000));

			// Mint fungible asset
			assert_ok!(mint_asset_on_asset_reserve(xUSD, ALICE, MINT_AMOUNT));
			assert_eq!(statemine_runtime::Assets::balance(xUSD, &ALICE), MINT_AMOUNT);
			statemine_runtime::System::events()
				.iter()
				.for_each(|r| println!(">>> {:?}", r.event));
			statemine_runtime::System::assert_has_event(
				pallet_xcm::Event::SupportedVersionChanged(kreivo_remote.clone(), 2).into(),
			);
		});

		// 5) Sends Transact as Superuser to Asset Reserve Parachain to set the asset as
		// sufficient
		KusamaNet::execute_with(|| {
			let set_asset_sufficient_call = statemine_runtime::RuntimeCall::Assets(pallet_assets::Call::<
				statemine_runtime::Runtime,
			>::force_asset_status {
				id: Compact(xUSD),
				owner: ALICE.into(),
				issuer: ALICE.into(),
				admin: ALICE.into(),
				freezer: ALICE.into(),
				min_balance: ASSET_MIN_BALANCE,
				is_sufficient: true,
				is_frozen: false,
			});

			assert_ok!(kusama_runtime::XcmPallet::send_xcm(
				Here,
				Parachain(ASSET_RESERVE_PARA_ID),
				Xcm(vec![Transact {
					origin_type: OriginKind::Superuser,
					require_weight_at_most: 1_000_000_000,
					call: set_asset_sufficient_call.encode().into(),
				}]),
			));
		});

		// 6) Check the events on Asset Reserve Parachain and validated the XCM
		// supported version is 2
		AssetReserveParachain::execute_with(|| {
			statemine_runtime::System::events()
				.iter()
				.for_each(|r| println!(">>> {:?}", r.event));
			statemine_runtime::System::assert_has_event(
				pallet_xcm::Event::SupportedVersionChanged(kreivo_remote.clone(), 2).into(),
			);
		});

		let mut beneficiary_balance = 0;
		// 7) Create derivative asset on Trappist Parachain
		// 8) Sets the asset as sufficient on Trappist	Parachain
		TrappistParachain::execute_with(|| {
			/* 			let statemine_sovereign_account = parachains::sovereign_account(ASSET_RESERVE_PARA_ID);

			assert_ok!(kreivo_runtime::Balances::transfer(
				kreivo_runtime::RuntimeOrigin::signed(BOB),
				MultiAddress::Id(statemine_sovereign_account.clone()),
				1_000_000_000_000
			)); */

			// Create derivative asset on Trappist Parachain
			assert_ok!(create_derivative_asset_on_trappist(
				txUSD,
				ALICE.into(),
				ASSET_MIN_BALANCE
			));

			let set_asset_sufficient_call = kreivo_runtime::RuntimeCall::Assets(pallet_assets::Call::<
				kreivo_runtime::Runtime,
			>::force_asset_status {
				id: Compact(txUSD),
				owner: ALICE.into(),
				issuer: ALICE.into(),
				admin: ALICE.into(),
				freezer: ALICE.into(),
				min_balance: ASSET_MIN_BALANCE,
				is_sufficient: true,
				is_frozen: false,
			});

			assert_ok!(kreivo_runtime::Sudo::sudo(
				kreivo_runtime::RuntimeOrigin::signed(ALICE),
				Box::new(set_asset_sufficient_call),
			));

			// Map derivative asset (txUSD) to multi-location (xUSD within Assets pallet on
			// Reserve Parachain) via Asset Registry
			assert_ok!(register_reserve_asset_on_trappist(ALICE, txUSD, xUSD));
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
			// Check beneficiary balance
			beneficiary_balance = kreivo_runtime::Assets::balance(txUSD, &ALICE);
		});

		const AMOUNT: u128 = 20_000_000_000;
		// 8) Fund Trappist sovereign account on Reserve Parachain
		// 9) Sends XCM to Trappist Parachain to reserve-transfer an asset to Trappist
		// Parachain
		AssetReserveParachain::execute_with(|| {
			let kreivo_sovereign_account = parachains::sovereign_account(KREIVO_PARA_ID);

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
						network: Any,
						id: ALICE.into()
					})
					.into()
					.into()
				),
				Box::new((X2(PalletInstance(50.into()), GeneralIndex(xUSD as u128)), AMOUNT).into()),
				0,
				WeightLimit::Unlimited,
			));

			assert_eq!(
				statemine_runtime::Assets::balance(xUSD, &kreivo_sovereign_account),
				AMOUNT
			);
		});

		// 10) Checks on Trappist Parachain that the asset was received
		const EST_FEES: u128 = 1_600_000_000 * 10;
		TrappistParachain::execute_with(|| {
			// Ensure beneficiary account balance increased
			let current_balance = kreivo_runtime::Assets::balance(txUSD, &ALICE);
			println!(
				"Reserve-transfer: initial balance {} transfer amount {} current balance {} estimated fees {} actual fees {}",
				beneficiary_balance.separate_with_commas(),
				AMOUNT.separate_with_commas(),
				current_balance.separate_with_commas(),
				EST_FEES.separate_with_commas(),
				(beneficiary_balance + AMOUNT - current_balance).separate_with_commas()
			);
			parachains::assert_balance(current_balance, beneficiary_balance + AMOUNT, EST_FEES);
		});
	}
}
