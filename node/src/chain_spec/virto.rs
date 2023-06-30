use crate::chain_spec::{get_account_id_from_seed, get_collator_keys_from_seed, Extensions, SAFE_XCM_VERSION};
use cumulus_primitives_core::ParaId;
use hex_literal::hex;

use sc_service::ChainType;

use sp_core::{crypto::UncheckedInto, sr25519};
use virto_runtime::{
	constants::currency::EXISTENTIAL_DEPOSIT, AccountId, AuraId, BalancesConfig, GenesisConfig, SessionConfig,
	SessionKeys, SudoConfig, SystemConfig,
};

const DEFAULT_PROTOCOL_ID: &str = "virto";

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<virto_runtime::GenesisConfig, Extensions>;

const VIRTO_PARA_ID: u32 = 2000;

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we
/// have just one key).
fn session_keys(aura: AuraId) -> SessionKeys {
	SessionKeys { aura }
}

pub fn virto_polkadot_chain_spec_local() -> ChainSpec {
	// Give your kreivo currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "DOT".into());
	properties.insert("tokenDecimals".into(), 10.into());
	properties.insert("ss58Format".into(), 1.into());

	ChainSpec::from_genesis(
		// Name
		"Virto Development",
		// ID
		"virto_dev",
		ChainType::Development,
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
				],
				VIRTO_PARA_ID.into(),
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
			para_id: VIRTO_PARA_ID,
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
			code: virto_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		parachain_info: virto_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: virto_runtime::CollatorSelectionConfig {
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
		polkadot_xcm: virto_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
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

pub fn virto_polkadot_chain_spec() -> ChainSpec {
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "DOT".into());
	properties.insert("tokenDecimals".into(), 10.into());
	properties.insert("ss58Format".into(), 1.into());

	ChainSpec::from_genesis(
		// Name
		"Virto",
		// ID
		"virto",
		ChainType::Live,
		move || {
			virto_live_genesis(
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
				VIRTO_PARA_ID.into(),
			)
		},
		vec![],
		None,
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "polkadot".into(),
			para_id: VIRTO_PARA_ID,
		},
	)
}

fn virto_live_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			code: virto_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.chain(std::iter::once(root_key.clone()))
				.map(|k| {
					if k == root_key {
						(k, 1_000_000_000_000_000_000)
					} else {
						(k, 1_500_000_000_000_000_000)
					}
				})
				.collect(),
		},
		parachain_info: virto_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: virto_runtime::CollatorSelectionConfig {
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
		polkadot_xcm: virto_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
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
