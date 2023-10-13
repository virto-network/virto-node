use super::*;
use crate::{Event, GovernanceStrategy, Info};
use sp_runtime::BoundedVec;

mod apply {
	use super::*;

	#[test]
	fn fails_if_community_already_exists() {
		new_test_ext().execute_with(|| {
			// Simulate a pre-existing community
			Info::<Test>::insert(
				COMMUNITY,
				CommunityInfo {
					state: CommunityState::Awaiting,
				},
			);
			GovernanceStrategy::<Test>::insert(COMMUNITY, CommunityGovernanceStrategy::AdminBased(COMMUNITY_ADMIN));

			assert_ok!(Communities::do_insert_member(&COMMUNITY, &COMMUNITY_ADMIN));

			// Should fail adding the community
			assert_noop!(
				Communities::apply(RuntimeOrigin::signed(COMMUNITY_ADMIN), COMMUNITY),
				Error::CommunityAlreadyExists
			);
		});
	}

	#[test]
	fn fails_if_not_enough_funds_to_take_deposit() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				Communities::apply(RuntimeOrigin::signed(COMMUNITY_ADMIN), COMMUNITY),
				DispatchError::Arithmetic(ArithmeticError::Underflow)
			);
		});
	}

	#[test]
	fn it_works() {
		new_test_ext().execute_with(|| {
			setup();

			System::assert_last_event(
				Event::CommunityCreated {
					id: COMMUNITY,
					who: COMMUNITY_ADMIN,
				}
				.into(),
			);
		});
	}
}

mod set_metadata {
	use super::*;

	#[test]
	fn fails_if_bad_origin() {
		new_test_ext().execute_with(|| {
			setup();

			// Fail if trying to call from unsigned origin
			assert_noop!(
				Communities::set_metadata(RuntimeOrigin::none(), COMMUNITY, None, None, None, None),
				DispatchError::BadOrigin
			);
			// Fail if trying to call from non-admin
			assert_noop!(
				Communities::set_metadata(
					RuntimeOrigin::signed(COMMUNITY_ADMIN + 1),
					COMMUNITY,
					None,
					None,
					None,
					None
				),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn works_inserts_default_metadata() {
		new_test_ext().execute_with(|| {
			setup();

			// Receives metadata information from admin
			assert_ok!(Communities::set_metadata(
				RuntimeOrigin::signed(COMMUNITY_ADMIN),
				COMMUNITY,
				Some(BoundedVec::truncate_from(b"Virto Network".to_vec())),
				None,
				None,
				None
			));

			let community_metadata =
				Communities::metadata(COMMUNITY).expect("We've already asserted that the insertion is succesful; qed");

			assert_eq!(
				community_metadata,
				crate::types::CommunityMetadata {
					name: BoundedVec::truncate_from(b"Virto Network".to_vec()),
					description: BoundedVec::new(),
					urls: BoundedVec::new(),
					locations: BoundedVec::new()
				}
			);

			// Receives metadata information from root
			assert_ok!(Communities::set_metadata(
				RuntimeOrigin::root(),
				COMMUNITY,
				None,
				Some(BoundedVec::truncate_from(b"A community of awesome builders".to_vec())),
				None,
				None
			));

			let community_metadata =
				Communities::metadata(COMMUNITY).expect("We've already asserted that the insertion is succesful; qed");

			assert_eq!(
				community_metadata,
				types::CommunityMetadata {
					name: BoundedVec::truncate_from(b"Virto Network".to_vec()),
					description: BoundedVec::truncate_from(b"A community of awesome builders".to_vec()),
					urls: BoundedVec::new(),
					locations: BoundedVec::new()
				}
			);
		});
	}
}
