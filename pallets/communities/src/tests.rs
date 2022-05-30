use crate::{mock::*, Communities as CommunitiesStorage, Community, CommunityId};
use frame_support::{assert_noop, assert_ok};

type Error = crate::Error<Test>;

fn last_event() -> Event {
	System::events().pop().expect("Event expected").event
}

#[test]
fn test_register_works() {
	new_test_ext().execute_with(|| {
		let controller = 1;
		let community_id = CommunityId {
			base: 100,
			category: 100,
			instance: 100,
		};

		// should be able to register a new community
		assert_ok!(Communities::register(
			Origin::signed(controller),
			community_id.clone(),
			vec![1u8; 10].try_into().unwrap()
		));

		assert_eq!(
			last_event(),
			crate::Event::<Test>::CommunityRegistered(community_id.clone()).into()
		);

		// should be stored correctly
		assert_eq!(
			CommunitiesStorage::<Test>::get((community_id.base, community_id.category, community_id.instance)),
			Some(Community {
				controller,
				population: Default::default(),
				domain_name: vec![1u8; 10].try_into().unwrap()
			})
		);

		// should not allow duplicate community ids
		assert_noop!(
			Communities::register(
				Origin::signed(controller),
				community_id.clone(),
				vec![1u8; 10].try_into().unwrap()
			),
			Error::InvalidCommunityId
		);
	});
}
