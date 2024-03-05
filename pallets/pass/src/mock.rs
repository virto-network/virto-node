use crate as pallet_pass;
use frame_support::traits::ConstU64;
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, ConstU32, IdentityLookup},
	BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

type AccountId = u64;

frame_support::parameter_types! {
	pub const StatemineParaIdInfo: u32 = 1000u32;
	pub const StatemineAssetsInstanceInfo: u8 = 50u8;
	pub const StatemineAssetIdInfo: u128 = 1u128;
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Pass: pallet_pass::{Pallet, Call, Storage, Event<T>},
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl pallet_pass::Config for Test {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

impl pallet_balances::Config for Test {
	type Balance = u64;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type MaxHolds = ();
	type RuntimeFreezeReason = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

// --- WebAuthN mock authenticator helper ---
use passkey_authenticator::{Authenticator, MemoryStore, MockUserValidationMethod};
use passkey_types::webauthn;

pub(crate) struct WebAuthN {
	client: passkey_client::Client<MemoryStore, MockUserValidationMethod, public_suffix::PublicSuffixList>,
	rt: tokio::runtime::Runtime,
	cred_id: Option<passkey_types::Bytes>,
	origin: url::Url,
}

impl WebAuthN {
	const ORIGIN: &'static str = "virto.dev";
	const USER: &'static str = "alice";

	pub fn new() -> Self {
		let mut user_mock = MockUserValidationMethod::new();
		user_mock
			.expect_is_verification_enabled()
			.returning(|| Some(true))
			.times(3);
		user_mock
			.expect_check_user_verification()
			.returning(|| Box::pin(async { true }))
			.times(2);
		user_mock.expect_is_presence_enabled().returning(|| true).times(1);

		let authenticator =
			Authenticator::new(passkey_types::ctap2::Aaguid::new_empty(), MemoryStore::new(), user_mock);

		let client = passkey_client::Client::new(authenticator);
		Self {
			client,
			rt: tokio::runtime::Builder::new_current_thread().build().unwrap(),
			cred_id: None,
			origin: url::Url::parse(&format!("https://{}", Self::ORIGIN)).unwrap(),
		}
	}

	pub fn create_credential(
		&mut self,
		challenge: &[u8],
	) -> webauthn::PublicKeyCredential<webauthn::AuthenticatorAttestationResponse> {
		self.rt.block_on(async {
			let cred = self
				.client
				.register(
					&self.origin,
					webauthn::CredentialCreationOptions {
						public_key: Self::credential_creation_options(challenge),
					},
					None,
				)
				.await
				.expect("credential");
			self.cred_id = Some(cred.raw_id.clone());
			cred
		})
	}

	pub fn assert(
		&mut self,
		challenge: &[u8],
	) -> webauthn::PublicKeyCredential<webauthn::AuthenticatorAssertionResponse> {
		self.rt.block_on(async {
			self.client
				.authenticate(
					&self.origin,
					webauthn::CredentialRequestOptions {
						public_key: self.credential_request_options(challenge),
					},
					None,
				)
				.await
				.expect("authenticated")
		})
	}

	fn user_id() -> passkey_types::Bytes {
		Vec::from(Self::USER.as_bytes()).into()
	}

	fn credential_creation_options(challenge: &[u8]) -> webauthn::PublicKeyCredentialCreationOptions {
		use coset::iana;
		webauthn::PublicKeyCredentialCreationOptions {
			rp: webauthn::PublicKeyCredentialRpEntity {
				id: Some(Self::ORIGIN.into()),
				name: Self::ORIGIN.into(),
			},
			user: webauthn::PublicKeyCredentialUserEntity {
				id: Self::user_id(),
				display_name: Self::USER.into(),
				name: Self::USER.into(),
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
			rp_id: Some(Self::ORIGIN.into()),
			allow_credentials: Some(vec![webauthn::PublicKeyCredentialDescriptor {
				ty: webauthn::PublicKeyCredentialType::PublicKey,
				id: self.cred_id.clone().unwrap(),
				transports: None,
			}]),
			user_verification: Default::default(),
			hints: Some(vec![webauthn::PublicKeyCredentialHints::Hybrid]),
			attestation: Default::default(),
			attestation_formats: Default::default(),
			extensions: Default::default(),
		}
	}
}
