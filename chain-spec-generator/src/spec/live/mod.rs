use super::*;

#[cfg(not(feature = "paseo"))]
pub fn chain_spec() -> Result<Box<dyn ChainSpec>, String> {
	Ok(Box::new(
		KreivoChainSpec::from_json_bytes(include_bytes!("./kreivo_kusama_chainspec.json").as_slice())
			.map_err(|_| "Could not find chainspec for kreivo")?,
	))
}

#[cfg(feature = "paseo")]
pub fn chain_spec() -> Result<Box<dyn ChainSpec>, String> {
	Ok(Box::new(
		KreivoChainSpec::from_json_bytes(include_bytes!("./kreivo_paseador_chainspec.json").as_slice())
			.map_err(|_| "Could not find chainspec for kreivo")?,
	))
}
