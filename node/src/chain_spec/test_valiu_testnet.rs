use crate::chain_spec::{
    account_id_from_ss58, properties, public_key_from_ss58, ChainSpec, GenesisConfigBuilder,
};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::sr25519;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use vln_runtime::WASM_BINARY;

const _1_AUTO_SS58: &str = "5HSxWDrQCtoM49VWt1HYaywGrSxwEJyn6AsbDVAoRMEevyU7";
const _1_GRANDPA_SS58: &str = "5CE3jbWckmKMaQnv8ECEscKJJNwCPJXrm8jPAjNf765qywvK";

const _2_AURA_SS58: &str = "5EaAVzfBcGWfzb4ZWMRcoxwweJk33yhzcPWmiurxmWVAPVhK";
const _2_GRANDPA_SS58: &str = "5FaC77dXnZkx4nm12omhyJc5xcUvHCJ8Z4MxJ9Gpa14WTqhu";

const _3_AURA_SS58: &str = "5EZeDVGEnG2SXKdHQPsimbJzB5tx18d1dMAocRvaPM2K2XCR";
const _3_GRANDPA_SS58: &str = "5ELYs2C7ePxsSNFZBHo5v5Rgq18fQzwepMzbdn43j5i724va";

pub fn test_valiu_testnet() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;

    let endowed_accounts = vec![
        account_id_from_ss58::<sr25519::Public>(_1_AUTO_SS58)?
    ];

    let initial_authorities = vec![
        (
            public_key_from_ss58::<AuraId>(_1_AUTO_SS58)?,
            public_key_from_ss58::<GrandpaId>(_1_GRANDPA_SS58)?,
        ),
        (
            public_key_from_ss58::<AuraId>(_2_AURA_SS58)?,
            public_key_from_ss58::<GrandpaId>(_2_GRANDPA_SS58)?,
        ),
        (
            public_key_from_ss58::<AuraId>(_3_AURA_SS58)?,
            public_key_from_ss58::<GrandpaId>(_3_GRANDPA_SS58)?,
        )
    ];

    let sudo_key = account_id_from_ss58::<sr25519::Public>(_1_AUTO_SS58)?;

    Ok(ChainSpec::from_genesis(
        "Test Valiu Testnet",
        "test_valiu_testnet",
        ChainType::Live,
        move || {
            GenesisConfigBuilder {
                endowed_accounts: &endowed_accounts,
                initial_authorities: &initial_authorities,
                sudo_key: sudo_key.clone(),
                wasm_binary,
            }
            .build()
        },
        vec![],
        None,
        Some("test_valiu"),
        Some(properties()),
        None,
    ))
}
