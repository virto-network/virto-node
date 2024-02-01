use super::*;
use crate::{
	types::{CommunityInfo, CommunityMetadata},
	Event, Info,
};
use frame_support::assert_noop;
use frame_system::RawOrigin::Root;
use sp_runtime::{BoundedVec, DispatchError};

mod create {
	use super::*;

	#[test]
	fn fails_if_community_already_exists() {
		new_test_ext(&[], &[]).execute_with(|| {
			// Simulate a pre-existing community
			Info::<Test>::insert(COMMUNITY, CommunityInfo::default());

			// Should fail adding the community
			assert_noop!(
				Communities::create(Root.into(), COMMUNITY_ORIGIN, COMMUNITY),
				Error::CommunityAlreadyExists
			);
		});
	}

	#[test]
	fn it_works() {
		new_test_ext(&[], &[]).execute_with(|| {
			System::assert_has_event(
				Event::CommunityCreated {
					id: COMMUNITY,
					origin: COMMUNITY_ORIGIN,
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
		new_test_ext(&[], &[]).execute_with(|| {
			// Fail if trying to call from unsigned origin
			assert_noop!(
				Communities::set_metadata(RuntimeOrigin::none(), COMMUNITY, None, None, None),
				DispatchError::BadOrigin
			);

			// Fail if trying to call from non-admin
			const NON_ADMIN: AccountId = AccountId::new([0; 32]);
			assert_noop!(
				Communities::set_metadata(RuntimeOrigin::signed(NON_ADMIN), COMMUNITY, None, None, None),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn works_inserts_default_metadata() {
		new_test_ext(&[], &[]).execute_with(|| {
			assert_ok!(Communities::set_metadata(
				Root.into(),
				COMMUNITY,
				Some(BoundedVec::truncate_from(b"Virto Network".to_vec())),
				None,
				None,
			));

			let community_metadata = Communities::metadata(COMMUNITY);

			assert_eq!(
				community_metadata,
				crate::types::CommunityMetadata {
					name: BoundedVec::truncate_from(b"Virto Network".to_vec()),
					description: BoundedVec::new(),
					main_url: BoundedVec::new(),
				}
			);

			assert_ok!(Communities::set_metadata(
				Root.into(),
				COMMUNITY,
				None,
				Some(BoundedVec::truncate_from(b"A community of awesome builders".to_vec())),
				None,
			));

			let community_metadata = Communities::metadata(COMMUNITY);

			assert_eq!(
				community_metadata,
				CommunityMetadata {
					name: BoundedVec::truncate_from(b"Virto Network".to_vec()),
					description: BoundedVec::truncate_from(b"A community of awesome builders".to_vec()),
					main_url: BoundedVec::new(),
				}
			);
		});
	}
}
