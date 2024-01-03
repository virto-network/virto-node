use crate::chain_spec::{get_account_id_from_seed, get_collator_keys_from_seed, Extensions, SAFE_XCM_VERSION};
use cumulus_primitives_core::ParaId;

use hex_literal::hex;
use kreivo_runtime::{
	constants::currency::EXISTENTIAL_DEPOSIT, AccountId, AuraId, BalancesConfig, RuntimeGenesisConfig, SessionConfig,
	SessionKeys, SudoConfig, SystemConfig,
};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

const DEFAULT_PROTOCOL_ID: &str = "kreivo";

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<kreivo_runtime::RuntimeGenesisConfig, Extensions>;

const KREIVO_PARA_ID: u32 = 2281;

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we
/// have just one key).
fn session_keys(aura: AuraId) -> SessionKeys {
	SessionKeys { aura }
}

pub fn kreivo_rococo_chain_spec_local() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "KSM".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 2.into());

	ChainSpec::builder(
		kreivo_runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
		Extensions {
			relay_chain: "rococo-local".into(),
			// You MUST set this to the correct network!
			para_id: KREIVO_PARA_ID.into(),
		},
	)
	.with_name("Kreivo Local-Rococo Local")
	.with_id("kreivo_rococo_local")
	.with_chain_type(ChainType::Local)
	.with_genesis_config_patch(testnet_genesis(
		// initial collators.
		vec![
			(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_collator_keys_from_seed::<AuraId>("Alice"),
			),
			(
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_collator_keys_from_seed::<AuraId>("Bob"),
			),
		],
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
			get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		KREIVO_PARA_ID.into(),
	))
	.build()
}

pub fn kreivo_kusama_chain_spec_local() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "KSM".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 2.into());

	ChainSpec::builder(
		kreivo_runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
		Extensions {
			relay_chain: "rococo-local".into(),
			// You MUST set this to the correct network!
			para_id: KREIVO_PARA_ID.into(),
		},
	)
	.with_name("Kreivo Local-Rococo Local")
	.with_id("kreivo_rococo_local")
	.with_chain_type(ChainType::Local)
	.with_genesis_config_patch(testnet_genesis(
		// initial collators.
		vec![
			(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_collator_keys_from_seed::<AuraId>("Alice"),
			),
			(
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_collator_keys_from_seed::<AuraId>("Bob"),
			),
		],
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
			get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		KREIVO_PARA_ID.into(),
	))
	.build()
}

pub fn local_testnet_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "UNIT".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	#[allow(deprecated)]
	ChainSpec::builder(
		kreivo_runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
		Extensions {
			relay_chain: "rococo-local".into(),
			// You MUST set this to the correct network!
			para_id: KREIVO_PARA_ID,
		},
	)
	.with_name("Local Testnet")
	.with_id("local_testnet")
	.with_chain_type(ChainType::Local)
	.with_genesis_config_patch(testnet_genesis(
		// initial collators.
		vec![
			(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_collator_keys_from_seed::<AuraId>("Alice"),
			),
			(
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_collator_keys_from_seed::<AuraId>("Bob"),
			),
		],
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
			get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		KREIVO_PARA_ID.into(),
	))
	.with_protocol_id(DEFAULT_PROTOCOL_ID)
	.with_properties(properties)
	.build()
}

pub fn kreivo_kusama_chain_spec() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "UNIT".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	#[allow(deprecated)]
	ChainSpec::builder(
		kreivo_runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
		Extensions {
			relay_chain: "kusama".into(),
			// You MUST set this to the correct network!
			para_id: KREIVO_PARA_ID.into(),
		},
	)
	.with_name("Kreivo")
	.with_id("kreivo")
	.with_chain_type(ChainType::Local)
	.with_genesis_config_patch(testnet_genesis(
		// initial collators.
		vec![
			(
				hex!("203aec61cedbfa0cd23f183a972b8646794b9106e62d141c6af4fbbbe293847b").into(),
				hex!("203aec61cedbfa0cd23f183a972b8646794b9106e62d141c6af4fbbbe293847b").unchecked_into(),
			),
			(
				hex!("74d538cee938b3f988567a3d0ad4b0dd84735ceab8b51cdfb850ecf58accfd7e").into(),
				hex!("74d538cee938b3f988567a3d0ad4b0dd84735ceab8b51cdfb850ecf58accfd7e").unchecked_into(),
			),
		],
		vec![
			// This account will have root origin
			hex!("68170716ab7c6735dd0a1012045d9ea33891b5f6596cf97eb217d0962d86a518").into(),
			hex!("556d3b25d068997f358622cc0f9531e4175d0d10d8ae8511c091d61efc21f65c").into(),
			hex!("8a0b6ddc780dbeb1c943caeadc7d09d85b2dc5b74026153f7931e068390d4441").into(),
		],
		hex!("7b953019065b4342a4f1fcf62be8f3e83c8d15303b674fd7191e598f699e764f").into(),
		KREIVO_PARA_ID.into(),
	))
	.with_protocol_id(DEFAULT_PROTOCOL_ID)
	.with_properties(properties)
	.build()
}

fn testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	root: AccountId,
	id: ParaId,
) -> serde_json::Value {
	serde_json::json!({
		"balances": {
			"balances": endowed_accounts.iter().cloned().map(|k| (k, 1u64 << 60)).collect::<Vec<_>>(),
		},
		"parachainInfo": {
			"parachainId": id,
		},
		"collatorSelection": {
			"invulnerables": invulnerables.iter().cloned().map(|(acc, _)| acc).collect::<Vec<_>>(),
			"candidacyBond": EXISTENTIAL_DEPOSIT * 16,
		},
		"session": {
			"keys": invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						session_keys(aura), // session keys
					)
				})
			.collect::<Vec<_>>(),
		},
		"polkadotXcm": {
			"safeXcmVersion": Some(SAFE_XCM_VERSION),
		},
		"sudo": { "key": Some(root) }
	})
}

fn genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	root: AccountId,
	id: ParaId,
) -> serde_json::Value {
	let balances: Vec<_> = endowed_accounts
		.iter()
		.cloned()
		.chain(std::iter::once(root.clone()))
		.map(|k| (k, 2_000_000_000_000_000u128))
		.collect();
	serde_json::json!({
		"balances": {
			"balances": balances
		},
		"parachainInfo": {
			"parachainId": id,
		},
		"collatorSelection": {
			"invulnerables": invulnerables.iter().cloned().map(|(acc, _)| acc).collect::<Vec<_>>(),
			"candidacyBond": EXISTENTIAL_DEPOSIT * 16,
		},
		"session": {
			"keys": invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						session_keys(aura), // session keys
					)
				})
			.collect::<Vec<_>>(),
		},
		"polkadotXcm": {
			"safeXcmVersion": Some(SAFE_XCM_VERSION),
		},
		"sudo": { "key": Some(root) },
	})
}
