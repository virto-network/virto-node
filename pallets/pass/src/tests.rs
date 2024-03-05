use crate::mock::new_test_ext;

mod foo {
	use super::*;
	use crate::mock;

	#[test]
	fn create_account() {
		let mut auth = mock::WebAuthN::new();
		new_test_ext().execute_with(|| {
			let cred = auth.create_credential(b"challenge");

			println!("{:?}", cred);
			assert_eq!(true, false);
		});
	}

	#[test]
	fn validate_webauthn_signed_payload() {
		let mut auth = mock::WebAuthN::new();
		auth.create_credential(b"challenge");
		new_test_ext().execute_with(|| {
			let res = auth.assert(b"challenge");
			println!("{res:?}");
			assert_eq!(true, false);
		})
	}
}
