use crate::mock::new_test_ext;
// use frame_support::{assert_noop, assert_ok};

mod foo {
	use super::*;

	#[test]
	fn register_reserve_asset_works() {
		new_test_ext().execute_with(|| {
			assert_eq!(true, false);
		});
	}
}
