use super::*;

#[cfg(not(feature = "paseo"))]
pub fn chain_spec() -> ChainSpec {
	ChainSpec::from_json_bytes(include_bytes!("./kreivo_kusama_chainspec.json").as_slice()).unwrap()
}

#[cfg(feature = "paseo")]
pub fn chain_spec() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "KSM".into());
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
	.with_name("Local Testnet")
	.with_id("local_testnet")
	.with_chain_type(ChainType::Local)
	.with_genesis_config_patch(local_genesis(
		KREIVO_PARA_ID.into(),
		// initial collators.
		vec![
			// TODO: @olanod insert collators here
			// (
			// 	get_account_id_from_seed::<sr25519::Public>("Alice"),
			// 	get_collator_keys_from_seed("Alice"),
			// ),
		],
		vec![],
		// EvoLanodoqDsgHb98Ymbu41uXXKfCPDKxeM6dXHyJ2JoVus
		[
			0x68, 0x17, 0x07, 0x16, 0xab, 0x7c, 0x67, 0x35, 0xdd, 0x0a, 0x10, 0x12, 0x04, 0x5d, 0x9e, 0xa3, 0x38, 0x91,
			0xb5, 0xf6, 0x59, 0x6c, 0xf9, 0x7e, 0xb2, 0x17, 0xd0, 0x96, 0x2d, 0x86, 0xa5, 0x18,
		]
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
