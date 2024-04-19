use super::*;
use crate::{types::CommunityInfo, Event, Info};
use frame_support::assert_noop;
use frame_system::RawOrigin::Root;

mod create {
	use super::*;

	const COMMUNITY_B: CommunityId = 2;
	const COMMUNITY_B_ORIGIN: OriginCaller = OriginCaller::Communities(crate::Origin::<Test>::new(COMMUNITY_B));

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

	#[test]
	fn takes_deposit_if_permissionlessly_creating_community() {
		new_test_ext(&[], &[]).execute_with(|| {
			const ALICE: AccountId = AccountId::new([1; 32]);
			assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), ALICE, 15));

			assert_ok!(Communities::create(
				RuntimeOrigin::signed(ALICE),
				COMMUNITY_B_ORIGIN,
				COMMUNITY_B
			));

			System::assert_has_event(
				Event::CommunityCreated {
					id: COMMUNITY_B,
					origin: COMMUNITY_B_ORIGIN,
				}
				.into(),
			);
			assert_eq!(Balances::free_balance(ALICE), 5);
		});
	}
}
