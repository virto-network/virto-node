use crate::chain_spec::{
    authority_keys_from_seed, get_account_id_from_seed, properties, ChainSpec, GenesisConfigBuilder,
};
use sc_service::ChainType;
use sp_core::sr25519;
use vln_runtime::WASM_BINARY;

pub fn local_testnet() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        "Local Testnet",
        "local_testnet",
        ChainType::Local,
        move || {
            GenesisConfigBuilder {
                endowed_accounts: &[
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
                initial_authorities: &[
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                ],
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
