use super::*;

#[cfg(not(feature = "paseo"))]
pub fn chain_spec() -> ChainSpec {
	ChainSpec::from_json_bytes(include_bytes!("./kreivo_kusama_chainspec.json").as_slice()).unwrap()
}

#[cfg(feature = "paseo")]
use sp_core::crypto::Ss58Codec;

#[cfg(feature = "paseo")]
pub fn chain_spec() -> ChainSpec {
	ChainSpec::from_json_bytes(include_bytes!("./kreivo_paseador_chainspec.json").as_slice()).unwrap()
}
