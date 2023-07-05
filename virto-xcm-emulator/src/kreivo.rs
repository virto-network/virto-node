use integration_tests_common::constants::{accounts, collators, XCM_V3};
use sp_core::{sr25519, storage::Storage};
use sp_runtime::BuildStorage;
use xcm_emulator::get_account_id_from_seed;

const SAFE_XCM_VERSION: u32 = XCM_V3;

pub const PARA_ID: u32 = 2000;
pub use kreivo_runtime::EXISTENTIAL_DEPOSIT as ED;

pub fn genesis(para_id: u32) -> Storage {
	let genesis_config = kreivo_runtime::GenesisConfig {
		system: kreivo_runtime::SystemConfig {
			code: kreivo_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: kreivo_runtime::BalancesConfig {
			balances: accounts::init_balances()
				.iter()
				.cloned()
				.map(|k| (k, ED * 4096))
				.collect(),
		},
		parachain_info: kreivo_runtime::ParachainInfoConfig {
			parachain_id: para_id.into(),
		},
		collator_selection: kreivo_runtime::CollatorSelectionConfig {
			invulnerables: collators::invulnerables().iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: ED * 16,
			..Default::default()
		},
		session: kreivo_runtime::SessionConfig {
			keys: collators::invulnerables()
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                          // account id
						acc,                                  // validator id
						kreivo_runtime::SessionKeys { aura }, // session keys
					)
				})
				.collect(),
		},
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		polkadot_xcm: kreivo_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
		sudo: kreivo_runtime::SudoConfig {
			key: Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
		},
		treasury: Default::default(),
		assets: Default::default(),
		transaction_payment: Default::default(),
		lockdown_mode: Default::default(),
	};

	genesis_config.build_storage().unwrap()
}
