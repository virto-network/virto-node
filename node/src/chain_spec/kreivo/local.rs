use super::*;

#[cfg(not(feature = "paseo"))]
pub fn chain_spec() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "KSM".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 2.into());

	ChainSpec::builder(
		kreivo_runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
		Extensions {
			relay_chain: "kusama-local".into(),
			// You MUST set this to the correct network!
			para_id: KREIVO_PARA_ID,
		},
	)
	.with_name("Kreivo-Kusama Local")
	.with_id("kreivo_kusama_local")
	.with_chain_type(ChainType::Local)
	.with_protocol_id(DEFAULT_PROTOCOL_ID)
	.with_properties(properties)
	.with_genesis_config_patch(local_genesis(
		KREIVO_PARA_ID.into(),
		// initial collators.
		vec![
			(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_collator_keys_from_seed("Alice"),
			),
			(
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_collator_keys_from_seed("Bob"),
			),
		],
		vec![
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
	))
	.build()
}

#[cfg(feature = "paseo")]
pub fn chain_spec() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "PAS".into());
	properties.insert("tokenDecimals".into(), 10.into());
	properties.insert("ss58Format".into(), 1.into());

	ChainSpec::builder(
		kreivo_runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
		Extensions {
			relay_chain: "paseo-local".into(),
			// You MUST set this to the correct network!
			para_id: KREIVO_PARA_ID,
		},
	)
	.with_name("Kreivo de Paseo-Local")
	.with_id("kreivo_paseo_local")
	.with_chain_type(ChainType::Local)
	.with_properties(properties)
	.with_genesis_config_patch(local_genesis(
		KREIVO_PARA_ID.into(),
		// initial collators.
		vec![
			(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_collator_keys_from_seed("Alice"),
			),
			(
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_collator_keys_from_seed("Bob"),
			),
		],
		vec![
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
		get_account_id_from_seed::<sr25519::Public>("Alice"),
	))
	.build()
}

fn local_genesis(
	id: ParaId,
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	#[cfg(feature = "paseo")] sudo: AccountId,
) -> serde_json::Value {
	let mut config = serde_json::json!({}).as_object().expect("map given; qed").clone();

	#[cfg(feature = "paseo")]
	config.insert("sudo".into(), serde_json::json!({ "key": sudo }));
	config.insert(
		"balances".into(),
		serde_json::json!({
			"balances": endowed_accounts.iter().cloned().map(|k| (k, 1u64 << 60)).collect::<Vec<_>>()
		}),
	);
	config.insert(
		"parachainInfo".into(),
		serde_json::json!({
			"parachainId": id,
		}),
	);
	config.insert(
		"collatorSelection".into(),
		serde_json::json!({
			"invulnerables": invulnerables.iter().cloned().map(|(acc, _)| acc).collect::<Vec<_>>(),
			"candidacyBond": EXISTENTIAL_DEPOSIT * 16,
		}),
	);
	config.insert(
		"session".into(),
		serde_json::json!({
			"keys": invulnerables
				.into_iter()
				.map(|(acc, aura)| {
				(
					acc.clone(),        // account id
					acc,                // validator id
					session_keys(aura), // session keys
				)
				})
				.collect::<Vec<_>>(),
		}),
	);
	config.insert(
		"polkadotXcm".into(),
		serde_json::json!({
			"safeXcmVersion": Some(SAFE_XCM_VERSION),
		}),
	);

	config.into()
}
