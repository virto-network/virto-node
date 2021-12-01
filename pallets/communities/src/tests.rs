use crate::{mock::*, CommunityRegistry};
use frame_support::{assert_noop, assert_ok};
use virto_primitives::CommunityId;

#[test]
fn register_community_works() {
	new_test_ext().execute_with(|| {
		// Dispatch a register community extrinsic
		assert_ok!(Communities::register(
			Origin::signed(1),
			CommunityId { lower: 1u8, upper: 1u8, res: 1u16 },
			"test".into()
		));
		// Read pallet storage and assert an expected result.
		assert_eq!(CommunityRegistry::<Test>::get((1, 1, 1)), Some("test".into()));
	});
}
