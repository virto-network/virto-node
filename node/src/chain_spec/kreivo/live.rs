use super::*;

#[cfg(not(feature = "paseo"))]
pub fn chain_spec() -> ChainSpec {
	ChainSpec::from_json_bytes(include_bytes!("./kreivo_kusama_chainspec.json").as_slice()).unwrap()
}

#[cfg(feature = "paseo")]
use sp_core::crypto::Ss58Codec;

#[cfg(feature = "paseo")]
pub fn chain_spec() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "PAS".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	#[allow(deprecated)]
	ChainSpec::builder(
		kreivo_runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
		Extensions {
			relay_chain: "paseo".into(),
			// You MUST set this to the correct network!
			para_id: KREIVO_PARA_ID,
		},
	)
	.with_name("Kreivo de Paseo")
	.with_id("kreivo")
	.with_chain_type(ChainType::Live)
	.with_genesis_config_patch(local_genesis(
		KREIVO_PARA_ID.into(),
		// initial collators.
		vec![(
			sr25519::Public::from_string("Ets3sAp4f2odq8FLEV17CYHwec5KgMvNHfV4MVuiLoadjVp")
				.expect("valid ss58 given; qed")
				.into(),
			sr25519::Public::from_string("Ets3sAp4f2odq8FLEV17CYHwec5KgMvNHfV4MVuiLoadjVp")
				.expect("valid ss58 given; qed")
				.into(),
		)],
		vec![], // No endowment. Send actual assets from Relay
		sr25519::Public::from_string("EvoLanodoqDsgHb98Ymbu41uXXKfCPDKxeM6dXHyJ2JoVus")
			.expect("valid ss58 given; qed")
			.into(),
	))
	.with_protocol_id(DEFAULT_PROTOCOL_ID)
	.with_properties(properties)
	.build()
}

#[cfg(feature = "paseo")]
fn local_genesis(
	id: ParaId,
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	sudo: AccountId,
) -> serde_json::Value {
	serde_json::json!({
		"balances": {
			"balances": endowed_accounts.iter().cloned().map(|k| (k, 1u64 << 60)).collect::<Vec<_>>(),
		},
		"parachainInfo": {
			"parachainId": id,
		},
		"collatorSelection": {
			"invulnerables": invulnerables.iter().cloned().map(|(acc, _)| acc).collect::<Vec<_>>(),
			"candidacyBond": EXISTENTIAL_DEPOSIT * 16,
		},
		"session": {
			"keys": invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						session_keys(aura), // session keys
					)
				})
			.collect::<Vec<_>>(),
		},
		"polkadotXcm": {
			"safeXcmVersion": Some(SAFE_XCM_VERSION),
		},
		"sudo": {
			"key": sudo
		}
	})
}
