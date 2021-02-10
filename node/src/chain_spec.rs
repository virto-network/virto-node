pub(crate) mod dev;
pub(crate) mod local;
pub(crate) mod testnet;

use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::Ss58Codec, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use vln_runtime::{
    AccountId, AuraConfig, GenesisConfig, GrandpaConfig, Signature, SudoConfig, SystemConfig,
    TokensConfig,
};

type AccountPublic = <Signature as Verify>::Signer;
pub(crate) type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

struct GenesisConfigBuilder<'a> {
    initial_authorities: &'a [(AuraId, GrandpaId)],
    sudo_key: AccountId,
    wasm_binary: &'a [u8],
}

impl GenesisConfigBuilder<'_> {
    fn build(self) -> GenesisConfig {
        GenesisConfig {
            frame_system: Some(SystemConfig {
                code: self.wasm_binary.to_vec(),
                changes_trie_config: Default::default(),
            }),
            orml_tokens: Some(TokensConfig {
                endowed_accounts: vec![],
            }),
            pallet_aura: Some(AuraConfig {
                authorities: self
                    .initial_authorities
                    .iter()
                    .map(|x| (x.0.clone()))
                    .collect(),
            }),
            pallet_grandpa: Some(GrandpaConfig {
                authorities: self
                    .initial_authorities
                    .iter()
                    .map(|x| (x.1.clone(), 1))
                    .collect(),
            }),
            pallet_membership: Some(pallet_membership::GenesisConfig {
                members: vec![],
                phantom: Default::default(),
            }),
            pallet_sudo: Some(SudoConfig { key: self.sudo_key }),
        }
    }
}

fn account_id_from_ss58<T: Public>(ss58: &str) -> Result<AccountId, String>
where
    AccountPublic: From<T>,
{
    Ok(AccountPublic::from(public_key_from_ss58(ss58)?).into_account())
}

fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

fn get_account_id_from_seed<T: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<T::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<T>(seed)).into_account()
}

fn get_from_seed<T: Public>(seed: &str) -> <T::Pair as Pair>::Public {
    T::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

fn public_key_from_ss58<T: Public>(ss58: &str) -> Result<T, String> {
    Ss58Codec::from_string(ss58).map_err(|_| "Couldn't generate public key from ss58 string".into())
}
