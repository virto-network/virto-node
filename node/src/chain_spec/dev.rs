use crate::chain_spec::{
    authority_keys_from_seed, get_account_id_from_seed, properties, ChainSpec, GenesisConfigBuilder,
};
use sc_service::ChainType;
use sp_core::sr25519;
use vln_runtime::WASM_BINARY;

pub fn dev() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Development,
        move || {
            GenesisConfigBuilder {
                endowed_accounts: &[
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                ],
                initial_authorities: &[authority_keys_from_seed("Alice")],
                sudo_key: get_account_id_from_seed::<sr25519::Public>("Alice"),
                wasm_binary,
            }
            .build()
        },
        vec![],
        None,
        None,
        Some(properties()),
        None,
    ))
}
