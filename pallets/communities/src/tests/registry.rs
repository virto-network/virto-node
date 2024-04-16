use super::*;
use crate::{types::CommunityInfo, Event, Info};
use frame_support::assert_noop;
use frame_system::RawOrigin::Root;

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
