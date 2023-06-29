pub use codec::Encode;
pub mod kreivo;
pub use frame_support::{
	assert_ok, instances::Instance1, pallet_prelude::Weight, parameter_types, sp_io, sp_tracing,
	traits::fungibles::Inspect,
};
pub use integration_tests_common::{
	constants::{
		accounts::{ALICE, BOB},
		kusama,
		kusama::ED as KUSAMA_ED,
		penpal, statemine, PROOF_SIZE_THRESHOLD, REF_TIME_THRESHOLD, XCM_V3,
	},
	AccountId, KusamaReceiver, KusamaSender, PenpalKusamaReceiver, PenpalKusamaSender, StatemineReceiver,
	StatemineSender,
};
pub use parachains_common::{AuraId, Balance, BlockNumber, StatemintAuraId};
pub use polkadot_core_primitives::InboundDownwardMessage;
pub use sp_core::{sr25519, storage::Storage, Get};
use std::sync::Once;

use crate::kreivo::kreivo::{genesis as kreivo_genesis, PARA_ID as KREIVO_PARA_ID};
pub use xcm::{
	prelude::*,
	v3::{Error, NetworkId::Kusama as KusamaId},
};
use xcm_emulator::{
	assert_expected_events, bx, cumulus_pallet_dmp_queue, decl_test_networks, decl_test_parachains,
	decl_test_relay_chains, helpers::weight_within_threshold, NetworkComponent, Parachain, RelayChain, TestExt,
};
use xcm_executor::traits::Convert;

decl_test_relay_chains! {
	pub struct Kusama {
		genesis = kusama::genesis(),
		on_init = (),
		runtime = {
			Runtime: kusama_runtime::Runtime,
			RuntimeOrigin: kusama_runtime::RuntimeOrigin,
			RuntimeCall: kusama_runtime::RuntimeCall,
			RuntimeEvent: kusama_runtime::RuntimeEvent,
			MessageQueue: kusama_runtime::MessageQueue,
			XcmConfig: kusama_runtime::xcm_config::XcmConfig,
			SovereignAccountOf: kusama_runtime::xcm_config::SovereignAccountOf,
			System: kusama_runtime::System,
			Balances: kusama_runtime::Balances,
		},
		pallets_extra = {
			XcmPallet: kusama_runtime::XcmPallet,
		}
	}
}

decl_test_parachains! {
	// Kusama
	pub struct Kreivo {
		genesis = kreivo_genesis(KREIVO_PARA_ID),
		on_init = (),
		runtime = {
			Runtime: kreivo_runtime::Runtime,
			RuntimeOrigin: kreivo_runtime::RuntimeOrigin,
			RuntimeCall: kreivo_runtime::RuntimeCall,
			RuntimeEvent: kreivo_runtime::RuntimeEvent,
			XcmpMessageHandler: kreivo_runtime::XcmpQueue,
			DmpMessageHandler: kreivo_runtime::DmpQueue,
			LocationToAccountId: kreivo_runtime::xcm_config::LocationToAccountId,
			System: kreivo_runtime::System,
			Balances: kreivo_runtime::Balances,
			ParachainSystem: kreivo_runtime::ParachainSystem,
			ParachainInfo: kreivo_runtime::ParachainInfo,
		},
		pallets_extra = {
			PolkadotXcm: kreivo_runtime::PolkadotXcm,
			Assets: kreivo_runtime::Assets,
			AssetRegistry: kreivo_runtime::AssetRegistry,
			LockdownMode: kreivo_runtime::LockdownMode,
		}
	},
	pub struct Statemine {
		genesis = statemine::genesis(),
		on_init = (),
		runtime = {
			Runtime: statemine_runtime::Runtime,
			RuntimeOrigin: statemine_runtime::RuntimeOrigin,
			RuntimeCall: statemine_runtime::RuntimeCall,
			RuntimeEvent: statemine_runtime::RuntimeEvent,
			XcmpMessageHandler: statemine_runtime::XcmpQueue,
			DmpMessageHandler: statemine_runtime::DmpQueue,
			LocationToAccountId: statemine_runtime::xcm_config::LocationToAccountId,
			System: statemine_runtime::System,
			Balances: statemine_runtime::Balances,
			ParachainSystem: statemine_runtime::ParachainSystem,
			ParachainInfo: statemine_runtime::ParachainInfo,
		},
		pallets_extra = {
			PolkadotXcm: statemine_runtime::PolkadotXcm,
			Assets: statemine_runtime::Assets,
			ForeignAssets: statemine_runtime::Assets,
		}
	}
}

decl_test_networks! {
	pub struct KusamaMockNet {
		relay_chain = Kusama,
		parachains = vec![
			Kreivo,
			Statemine,
		],
	}
}

parameter_types! {
	// Kreivo
	pub KreivoSender: AccountId = Kreivo::account_id_of(ALICE);
	pub KreivoReceiver: AccountId = Kreivo::account_id_of(BOB);
}

pub fn assert_balance(actual: u128, expected: u128, fees: u128) {
	assert!(
		actual >= (expected - fees) && actual <= expected,
		"expected: {expected}, actual: {actual} fees: {fees}"
	)
}

#[cfg(test)]
mod tests;
