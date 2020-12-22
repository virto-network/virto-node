use crate::chain_spec::{
    account_id_from_ss58, properties, public_key_from_ss58, ChainSpec, GenesisConfigBuilder,
};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::sr25519;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use vln_runtime::WASM_BINARY;

const ODIN_AURA_SS58: &str = "5DodinBNh59Ew9bvRfoDmj1j1QXzPhvHqk5w6jpdAfq7DHZd";
const ODIN_GRANDPA_SS58: &str = "5CodinauszfC7FeAnugMk1eXiBCsrJmg4pE6RQN9GzQ7nEpb";

const THOR_AURA_SS58: &str = "5Cthor4p3N8XMhYRjsKta7nXv2FsBc8gmX7p9pDAoejEeAhR";
const THOR_GRANDPA_SS58: &str = "5GthorY858eiYyC48XPtL6fWDpmvv4tM6qaBRHuPc4sjER5V";

const LOKI_AURA_SS58: &str = "5CLokibKMfc9NfPWaEzEqkLAL2JkZLA3hZNGXxvSgCJkSgTQ";
const LOKI_GRANDPA_SS58: &str = "5ELokiYDQEjTGk1hDSKuVusyHyuYYa1e8JGEc7gfiY9f3eey";

pub fn chain_spec() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;

    let initial_authorities = vec![
        (
            public_key_from_ss58::<AuraId>(ODIN_AURA_SS58)?,
            public_key_from_ss58::<GrandpaId>(ODIN_GRANDPA_SS58)?,
        ),
        (
            public_key_from_ss58::<AuraId>(THOR_AURA_SS58)?,
            public_key_from_ss58::<GrandpaId>(THOR_GRANDPA_SS58)?,
        ),
        (
            public_key_from_ss58::<AuraId>(LOKI_AURA_SS58)?,
            public_key_from_ss58::<GrandpaId>(LOKI_GRANDPA_SS58)?,
        ),
    ];

    let sudo_key = account_id_from_ss58::<sr25519::Public>(ODIN_AURA_SS58)?;

    Ok(ChainSpec::from_genesis(
        "Testnet Testnet",
        "testnet_testnet",
        ChainType::Live,
        move || {
            GenesisConfigBuilder {
                initial_authorities: &initial_authorities,
                sudo_key: sudo_key.clone(),
                wasm_binary,
            }
            .build()
        },
        vec![],
        None,
        Some("testnet"),
        Some(properties()),
        None,
    ))
}
