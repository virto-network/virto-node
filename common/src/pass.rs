#[cfg(feature = "pass-client")]
use alloc::{string::String, vec, vec::Vec};
#[cfg(feature = "pass-client")]
use passkey_types::{webauthn, Bytes};
#[cfg(feature = "js")]
use wasm_bindgen::prelude::*;

/// UserPass helps creating and using pallet-pass compatible webauthn credentials on the client
#[cfg_attr(feature = "pass-client-js", wasm_bindgen)]
#[cfg(feature = "pass-client")]
pub struct UserPass {
	user: String,
}

#[cfg_attr(feature = "pass-client-js", wasm_bindgen)]
#[cfg(feature = "pass-client")]
impl UserPass {
	#[cfg_attr(feature = "pass-client-js", wasm_bindgen(constructor))]
	pub fn new(user: &str) -> Self {
		Self { user: user.into() }
	}

	fn credential_creation_options(&self, challenge: &[u8]) -> webauthn::PublicKeyCredentialCreationOptions {
		use coset::iana;
		const DISP_NAME: &str = "Virto Pass";
		webauthn::PublicKeyCredentialCreationOptions {
			rp: webauthn::PublicKeyCredentialRpEntity {
				id: None,
				name: DISP_NAME.into(),
			},
			user: webauthn::PublicKeyCredentialUserEntity {
				id: self.user_id(),
				display_name: self.user.clone(),
				name: self.user.to_lowercase(),
			},
			challenge: Vec::from(challenge).into(),
			pub_key_cred_params: vec![
				webauthn::PublicKeyCredentialParameters {
					ty: webauthn::PublicKeyCredentialType::PublicKey,
					alg: iana::Algorithm::EdDSA,
				},
				webauthn::PublicKeyCredentialParameters {
					ty: webauthn::PublicKeyCredentialType::PublicKey,
					alg: iana::Algorithm::ES256,
				},
			],
			timeout: None,
			exclude_credentials: Default::default(),
			authenticator_selection: Some(webauthn::AuthenticatorSelectionCriteria {
				authenticator_attachment: None,
				resident_key: Some(webauthn::ResidentKeyRequirement::Required),
				require_resident_key: true,
				user_verification: webauthn::UserVerificationRequirement::Required,
			}),
			hints: Some(vec![webauthn::PublicKeyCredentialHints::Hybrid]),
			attestation: Default::default(),
			attestation_formats: Default::default(),
			extensions: Default::default(),
		}
	}

	fn credential_request_options(&self, challenge: &[u8]) -> webauthn::PublicKeyCredentialRequestOptions {
		webauthn::PublicKeyCredentialRequestOptions {
			challenge: Vec::from(challenge).into(),
			timeout: None,
			rp_id: None,
			allow_credentials: None,
			user_verification: Default::default(),
			hints: Some(vec![webauthn::PublicKeyCredentialHints::Hybrid]),
			attestation: Default::default(),
			attestation_formats: Default::default(),
			extensions: Default::default(),
		}
	}

	fn user_id(&self) -> Bytes {
		Vec::from(self.user.as_bytes()).into()
	}
}

#[cfg(feature = "pass-client-js")]
#[cfg_attr(feature = "pass-client-js", wasm_bindgen)]
impl UserPass {
	pub async fn create_credential(
		&mut self,
		challenge: &[u8],
	) -> Option<webauthn::PublicKeyCredential<webauthn::AuthenticatorAttestationResponse>> {
		let opts = self.credential_creation_options(challenge);
		let opts = serde_wasm_bindgen::to_value(&opts).expect("serializable");
		let promise = web_sys::window()?
			.navigator()
			.credentials()
			.create_with_options(&opts.into())
			.ok()?;
		let fut: wasm_bindgen_futures::JsFuture = promise.into();
		let cred = fut.await;
		todo!()
	}

	pub async fn assert(
		&mut self,
		challenge: &[u8],
	) -> Option<webauthn::PublicKeyCredential<webauthn::AuthenticatorAssertionResponse>> {
		let opts = self.credential_request_options(challenge);
		let opts = serde_wasm_bindgen::to_value(&opts).expect("serializable");
		let promise = web_sys::window()?
			.navigator()
			.credentials()
			.get_with_options(&opts.into())
			.ok()?;
		let fut: wasm_bindgen_futures::JsFuture = promise.into();
		let cred = fut.await;
		todo!()
	}
}

type Bytes = Vec<u8>;

/// Runtime friendly version of
// #[derive(Debug, Clone, Deserialize, Serialize)]
// #[serde(rename_all = "camelCase")]
pub struct AttestationResponse {
	///
	// #[serde(rename = "clientDataJSON")]
	pub client_data_json: Bytes,
	///
	pub authenticator_data: Bytes,
	///
	// #[serde(skip_serializing_if = "Option::is_none")]
	pub public_key: Option<Bytes>,
	///
	// #[typeshare(serialized_as = "I54")] // because i64 fails for js
	pub public_key_algorithm: i64,
	///
	pub attestation_object: Bytes,
	///
	// #[serde(default, skip_serializing_if = "Option::is_none")]
	pub transports: Option<Vec<AuthenticatorTransport>>,
}

// #[derive(Debug, Deserialize, Serialize)]
pub struct AssertionResponse {
	/// This attribute contains the JSON serialization of [`CollectedClientData`] passed to the
	// #[serde(rename = "clientDataJSON")]
	pub client_data_json: Bytes,
	/// This attribute contains the authenticator data returned by the authenticator. See [`AuthenticatorData`].
	pub authenticator_data: Bytes,
	/// This attribute contains the raw signature returned from the authenticator.
	pub signature: Bytes,
	/// This attribute contains the user handle returned from the authenticator, or null if the authenticator did not return a user handle.
	// #[serde(default, skip_serializing_if = "Option::is_none")]
	pub user_handle: Option<Bytes>,
	/// This OPTIONAL attribute contains an attestation object
	// #[serde(default, skip_serializing_if = "Option::is_none")]
	pub attestation_object: Option<Bytes>,
}
