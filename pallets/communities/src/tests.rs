use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn register_community_works() {
	new_test_ext().execute_with(|| {
		// Dispatch a register community extrinsic
		assert_ok!(Communities::register(Origin::signed(1), 42, "test".into()));
		// Read pallet storage and assert an expected result.
		//assert_eq!(TemplateModule::something(), Some(42));
	});
}
