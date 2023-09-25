use crate::chain_spec::{get_account_id_from_seed, get_collator_keys_from_seed, Extensions, SAFE_XCM_VERSION};
use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use kreivo_runtime::{
	constants::currency::EXISTENTIAL_DEPOSIT, AccountId, AuraId, BalancesConfig, RuntimeGenesisConfig, SessionConfig,
	SessionKeys, SudoConfig, SystemConfig,
};
use sc_service::ChainType;
use sp_core::{crypto::UncheckedInto, sr25519};
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
	// Give your kreivo currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "KSM".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 2.into());

	ChainSpec::from_genesis(
		// Name
		"Kreivo Local-Rococo Local",
		// ID
		"kreivo_rococo_local",
		ChainType::Local,
		move || {
			testnet_genesis(
				// Initial collators.
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
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
				],
				KREIVO_PARA_ID.into(),
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some(DEFAULT_PROTOCOL_ID),
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: KREIVO_PARA_ID,
		},
	)
}

pub fn kreivo_kusama_chain_spec_local() -> ChainSpec {
	// Give your kreivo currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("ss58Format".into(), 2.into());
	properties.insert("tokenSymbol".into(), "KSM".into());
	properties.insert("tokenDecimals".into(), 12.into());

	ChainSpec::from_genesis(
		// Name
		"Kreivo Local-Kusama Local",
		// ID
		"kreivo-kusama-local",
		ChainType::Local,
		move || {
			testnet_genesis(
				// Initial collators.
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
				// Sudo account
				hex!("49daa32c7287890f38b7e1a8cd2961723d36d20baa0bf3b82e0c4bdda93b1c0a").into(),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
				],
				KREIVO_PARA_ID.into(),
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some(DEFAULT_PROTOCOL_ID),
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "kusama-local".into(), // You MUST set this to the correct network!
			para_id: KREIVO_PARA_ID,
		},
	)
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> RuntimeGenesisConfig {
	RuntimeGenesisConfig {
		system: SystemConfig {
			code: kreivo_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			_config: Default::default(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts
				.iter()
				.cloned()
				.chain(std::iter::once(root_key.clone()))
				.map(|k| (k, 2_000_000_000_000_000))
				.collect(),
		},
		parachain_info: kreivo_runtime::ParachainInfoConfig {
			parachain_id: id,
			_config: Default::default(),
		},
		collator_selection: kreivo_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: SessionConfig {
			keys: invulnerables
				.iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                // account id
						acc.clone(),                // validator id
						session_keys(aura.clone()), // session keys
					)
				})
				.collect(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		polkadot_xcm: kreivo_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			_config: Default::default(),
		},
		transaction_payment: Default::default(),
		lockdown_mode: Default::default(),
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		treasury: Default::default(),
		assets: Default::default(),
	}
}

pub fn kreivo_kusama_chain_spec() -> ChainSpec {
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("ss58Format".into(), 2.into());
	properties.insert("tokenSymbol".into(), "KSM".into());
	properties.insert("tokenDecimals".into(), 12.into());

	ChainSpec::from_genesis(
		// Name
		"Kreivo",
		// ID
		"kreivo",
		ChainType::Live,
		move || {
			kreivo_live_genesis(
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
				hex!("7b953019065b4342a4f1fcf62be8f3e83c8d15303b674fd7191e598f699e764f").into(),
				vec![
					// This account will have root origin
					hex!("68170716ab7c6735dd0a1012045d9ea33891b5f6596cf97eb217d0962d86a518").into(),
					hex!("556d3b25d068997f358622cc0f9531e4175d0d10d8ae8511c091d61efc21f65c").into(),
					hex!("8a0b6ddc780dbeb1c943caeadc7d09d85b2dc5b74026153f7931e068390d4441").into(),
				],
				KREIVO_PARA_ID.into(),
			)
		},
		vec![],
		None,
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "kusama".into(),
			para_id: KREIVO_PARA_ID,
		},
	)
}

fn kreivo_live_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> RuntimeGenesisConfig {
	RuntimeGenesisConfig {
		system: SystemConfig {
			code: kreivo_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			_config: Default::default(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.chain(std::iter::once(root_key.clone()))
				.map(|k| (k, 2_000_000_000_000_000))
				.collect(),
		},
		parachain_info: kreivo_runtime::ParachainInfoConfig {
			parachain_id: id,
			_config: Default::default(),
		},
		collator_selection: kreivo_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: SessionConfig {
			keys: invulnerables
				.iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                // account id
						acc.clone(),                // validator id
						session_keys(aura.clone()), // session keys
					)
				})
				.collect(),
		},
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		polkadot_xcm: kreivo_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			_config: Default::default(),
		},
		transaction_payment: Default::default(),
		lockdown_mode: Default::default(),
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		treasury: Default::default(),
		assets: Default::default(),
	}
}
