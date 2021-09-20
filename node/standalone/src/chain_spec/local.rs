use crate::chain_spec::{
    authority_keys_from_seed, get_account_id_from_seed, ChainSpec, GenesisConfigBuilder,
};
use sc_service::ChainType;
use sp_core::sr25519;
use virto_runtime::WASM_BINARY;

pub fn chain_spec() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        "Virto Local Testnet",
        "local",
        ChainType::Local,
        move || {
            GenesisConfigBuilder {
                initial_authorities: &[
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                ],
                endowed_accounts: vec![],
                sudo_key: get_account_id_from_seed::<sr25519::Public>("Alice"),
                wasm_binary,
            }
            .build()
        },
        vec![],
        None,
        None,
        None,
        None,
    ))
}
