use cumulus_primitives_core::ParaId;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use vln_runtime::{AccountId, Signature};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<vln_runtime::GenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
    /// The relay chain of the Parachain.
    pub relay_chain: String,
    /// The id of the Parachain.
    pub para_id: u32,
}

impl Extensions {
    /// Try to get the extension from the given `ChainSpec`.
    pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
        sc_chain_spec::get_extension(chain_spec.extensions())
    }
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn development_config(id: ParaId) -> ChainSpec {
    ChainSpec::from_genesis(
        // Name
        "VLN Development",
        // ID
        "dev",
        ChainType::Local,
        move || testnet_genesis(get_account_id_from_seed::<sr25519::Public>("Alice"), id),
        vec![],
        None,
        None,
        None,
        Extensions {
            relay_chain: "rococo-dev".into(),
            para_id: id.into(),
        },
    )
}

pub fn local_testnet_config(id: ParaId) -> ChainSpec {
    ChainSpec::from_genesis(
        // Name
        "VLN Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || testnet_genesis(get_account_id_from_seed::<sr25519::Public>("Alice"), id),
        vec![],
        None,
        None,
        None,
        Extensions {
            relay_chain: "rococo-local".into(),
            para_id: id.into(),
        },
    )
}

fn testnet_genesis(root_key: AccountId, id: ParaId) -> vln_runtime::GenesisConfig {
    vln_runtime::GenesisConfig {
        frame_system: vln_runtime::SystemConfig {
            code: vln_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
            changes_trie_config: Default::default(),
        },
        pallet_sudo: vln_runtime::SudoConfig {
            key: root_key.clone(),
        },
        parachain_info: vln_runtime::ParachainInfoConfig { parachain_id: id },
        orml_tokens_Instance1: vln_runtime::TokensConfig {
            endowed_accounts: vec![],
        },
        orml_tokens_Instance2: vln_runtime::CollateralConfig {
            endowed_accounts: vec![],
        },
        orml_tokens_Instance3: vln_runtime::ForeignTokensConfig {
            endowed_accounts: vec![],
        },
        pallet_membership: vln_runtime::WhitelistConfig {
            members: vec![root_key],
            phantom: Default::default(),
        },
    }
}
