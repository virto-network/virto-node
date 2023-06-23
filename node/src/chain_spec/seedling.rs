use crate::chain_spec::{get_account_id_from_seed, get_collator_keys_from_seed, Extensions, SAFE_XCM_VERSION};
use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use sc_service::ChainType;
use seedling_runtime::{
	constants::currency::EXISTENTIAL_DEPOSIT, AccountId, AuraId, BalancesConfig, GenesisConfig, SessionConfig,
	SessionKeys, SudoConfig, SystemConfig,
};
use sp_core::{crypto::UncheckedInto, sr25519};

/// Specialized `ChainSpec` for the seedling parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<seedling_runtime::GenesisConfig, Extensions>;

const DEFAULT_PROTOCOL_ID: &str = "seedling";
const SEEDLING_PARA_ID: u32 = 4265;

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we
/// have just one key).
fn session_keys(aura: AuraId) -> SessionKeys {
	SessionKeys { aura }
}

pub fn seedling_rococo_chain_spec_local() -> ChainSpec {
	// Give your seedling currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "ROC".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Seedling Local-Rococo Local",
		// ID
		"seedling_rococo_local",
		ChainType::Local,
		move || {
			testnet_genesis(
				// Initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed::<AuraId>("Bob"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Charlie"),
						get_collator_keys_from_seed::<AuraId>("Charlie"),
					),
				],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
				SEEDLING_PARA_ID.into(),
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
			para_id: SEEDLING_PARA_ID,
		},
	)
}

pub fn seedling_kusama_chain_spec_local() -> ChainSpec {
	// Give your seedling currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "KSM".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 2.into());

	ChainSpec::from_genesis(
		// Name
		"Seedling Local-Kusama Local",
		// ID
		"seedling_kusama_local",
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
				SEEDLING_PARA_ID.into(),
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
			para_id: SEEDLING_PARA_ID,
		},
	)
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			code: seedling_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 2_000_000_000_000_000))
				.collect(),
		},
		parachain_info: seedling_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: seedling_runtime::CollatorSelectionConfig {
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
		polkadot_xcm: seedling_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
		transaction_payment: Default::default(),
		lockdown_mode: Default::default(),
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
	}
}

pub fn seedling_rococo_chain_spec() -> ChainSpec {
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "ROC".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Seedling",
		// ID
		"seedling",
		ChainType::Live,
		move || {
			seedling_live_genesis(
				// initial collators.
				vec![
					(
						hex!("a064c3e568cb53296ded94162c21e907cc650dec7c340d5cf5b32115d9a88243").into(),
						hex!("a064c3e568cb53296ded94162c21e907cc650dec7c340d5cf5b32115d9a88243").unchecked_into(),
					),
					(
						hex!("14d17e6196c53317e4b6440334dfb4c7db2c68caa4c27d1ce800dc32fba61d5f").into(),
						hex!("14d17e6196c53317e4b6440334dfb4c7db2c68caa4c27d1ce800dc32fba61d5f").unchecked_into(),
					),
				],
				hex!("aada5995f2188a1f619d4d3629692cabebd220d6fe5f1249f768f8a4f01e7f21").into(),
				vec![
					// This account will have root origin
					hex!("aada5995f2188a1f619d4d3629692cabebd220d6fe5f1249f768f8a4f01e7f21").into(),
				],
				SEEDLING_PARA_ID.into(),
			)
		},
		vec![],
		None,
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "rococo".into(),
			para_id: SEEDLING_PARA_ID,
		},
	)
}

pub fn seedling_kusama_chain_spec() -> ChainSpec {
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "KSM".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 2.into());

	ChainSpec::from_genesis(
		// Name
		"Seedling",
		// ID
		"seedling",
		ChainType::Live,
		move || {
			seedling_live_genesis(
				// initial collators.
				vec![
					(
						hex!("441f1878b52468c7f4da41cec27dc13e6843c283a58d33485212ae48cf94fb3c").into(),
						hex!("441f1878b52468c7f4da41cec27dc13e6843c283a58d33485212ae48cf94fb3c").unchecked_into(),
					),
					(
						hex!("441f1878b52468c7f4da41cec27dc13e6843c283a58d33485212ae48cf94fb3c").into(),
						hex!("441f1878b52468c7f4da41cec27dc13e6843c283a58d33485212ae48cf94fb3c").unchecked_into(),
					),
				],
				hex!("441f1878b52468c7f4da41cec27dc13e6843c283a58d33485212ae48cf94fb3c").into(),
				vec![
					// This account will have root origin
					hex!("441f1878b52468c7f4da41cec27dc13e6843c283a58d33485212ae48cf94fb3c").into(),
					hex!("441f1878b52468c7f4da41cec27dc13e6843c283a58d33485212ae48cf94fb3c").into(),
					hex!("441f1878b52468c7f4da41cec27dc13e6843c283a58d33485212ae48cf94fb3c").into(),
					hex!("441f1878b52468c7f4da41cec27dc13e6843c283a58d33485212ae48cf94fb3c").into(),
					hex!("441f1878b52468c7f4da41cec27dc13e6843c283a58d33485212ae48cf94fb3c").into(),
					hex!("441f1878b52468c7f4da41cec27dc13e6843c283a58d33485212ae48cf94fb3c").into(),
				],
				SEEDLING_PARA_ID.into(),
			)
		},
		vec![],
		None,
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "kusama".into(),
			para_id: SEEDLING_PARA_ID,
		},
	)
}

fn seedling_live_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			code: seedling_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 2_000_000_000_000_000))
				.collect(),
		},
		parachain_info: seedling_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: seedling_runtime::CollatorSelectionConfig {
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
		polkadot_xcm: seedling_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
		transaction_payment: Default::default(),
		lockdown_mode: Default::default(),
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
	}
}
