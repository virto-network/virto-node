use hex_literal::hex;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use serde::{Deserialize, Serialize};
use sp_core::crypto::Ss58Codec;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use vln_runtime::{AccountId, AuraId, Signature};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<vln_runtime::GenesisConfig, Extensions>;

pub const PARA_ID: u32 = 2086u32;

const THOR_AURA_SS58: &str = "5Hgs2iwvqjFNdhQX5yXG4YJNy9CXWmFS1884VWLC353tmY8Q";
const ODIN_AURA_SS58: &str = "5GN3Ne9KDobS3NknsqPvvbFXBPtU9mcMBmpxAyX2bidSmmGK";

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

fn public_key_from_ss58<T: Public>(ss58: &str) -> T {
    Ss58Codec::from_string(ss58).unwrap()
}

pub fn development_config() -> ChainSpec {
    let mut properties = Properties::new();
    properties.insert("tokenSymbol".into(), "KSM".into());
    properties.insert("tokenDecimals".into(), 10.into());

    ChainSpec::from_genesis(
        // Name
        "VLN PC Dev",
        // ID
        "dev",
        ChainType::Local,
        move || {
            testnet_genesis(
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    get_from_seed::<AuraId>("Alice"),
                    get_from_seed::<AuraId>("Bob"),
                ],
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                ],
            )
        },
        vec![],
        None,
        None,
        Some(properties),
        Extensions {
            relay_chain: "rococo-local".into(),
            para_id: PARA_ID,
        },
    )
}

pub fn testnet_config() -> ChainSpec {
    let testnet_root_key: AccountId =
        hex!["b2c27cac9a4a7f6003cde27ef5b37a0245efdd202c3a6759130dd5c846ee285b"].into();
    ChainSpec::from_genesis(
        // Name
        "VLN PC",
        // ID
        "testnet",
        ChainType::Live,
        move || {
            testnet_genesis(
                testnet_root_key.clone(),
                vec![
                    public_key_from_ss58::<AuraId>(THOR_AURA_SS58),
                    public_key_from_ss58::<AuraId>(ODIN_AURA_SS58),
                ],
                vec![],
            )
        },
        vec![],
        None,
        None,
        None,
        Extensions {
            relay_chain: "rococo".into(),
            para_id: PARA_ID,
        },
    )
}

fn testnet_genesis(
    root_key: AccountId,
    initial_authorities: Vec<AuraId>,
    endowed_accounts: Vec<AccountId>,
) -> vln_runtime::GenesisConfig {
    vln_runtime::GenesisConfig {
        system: vln_runtime::SystemConfig {
            code: vln_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
            changes_trie_config: Default::default(),
        },
        sudo: vln_runtime::SudoConfig {
            key: root_key.clone(),
        },
        balances: vln_runtime::BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        },
        parachain_info: vln_runtime::ParachainInfoConfig {
            parachain_id: PARA_ID.into(),
        },
        aura: vln_runtime::AuraConfig {
            authorities: initial_authorities,
        },
        aura_ext: Default::default(),
        whitelist: vln_runtime::WhitelistConfig {
            members: vec![root_key],
            phantom: Default::default(),
        },
        parachain_system: Default::default(),
        tokens: vln_runtime::TokensConfig { balances: vec![] },
        collateral: vln_runtime::CollateralConfig { balances: vec![] },
    }
}
