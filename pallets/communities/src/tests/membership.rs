use super::*;

const COMMUNITY_MEMBER_1: u64 = 43;
const COMMUNITY_MEMBER_2: u64 = 44;
const COMMUNITY_NON_MEMBER: u64 = 45;

mod add_member {
	use super::*;

	#[test]
	fn fails_when_community_is_not_active() {
		new_test_ext().execute_with(|| {
			setup();

			assert_noop!(
				Communities::add_member(RuntimeOrigin::signed(COMMUNITY_ADMIN), COMMUNITY, COMMUNITY_MEMBER_1),
				Error::CommunityNotActive
			);
		});
	}

	#[test]
	fn fails_when_caller_not_a_member() {
		new_test_ext().execute_with(|| {
			setup();
			assert_ok!(Communities::do_force_complete_challenge(&COMMUNITY));

			assert_noop!(
				Communities::add_member(RuntimeOrigin::none(), COMMUNITY, COMMUNITY_MEMBER_1),
				DispatchError::BadOrigin
			);

			assert_noop!(
				Communities::add_member(
					RuntimeOrigin::signed(COMMUNITY_NON_MEMBER),
					COMMUNITY,
					COMMUNITY_MEMBER_1
				),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn adds_members() {
		new_test_ext().execute_with(|| {
			setup();
			assert_ok!(Communities::do_force_complete_challenge(&COMMUNITY));

			assert_noop!(
				Communities::add_member(RuntimeOrigin::none(), COMMUNITY, COMMUNITY_MEMBER_1),
				DispatchError::BadOrigin
			);

			// Successfully adds a member
			assert_ok!(Communities::add_member(
				RuntimeOrigin::signed(COMMUNITY_ADMIN),
				COMMUNITY,
				COMMUNITY_MEMBER_1
			));

			// Once a member, can add other members
			assert_ok!(Communities::add_member(
				RuntimeOrigin::signed(COMMUNITY_MEMBER_1),
				COMMUNITY,
				COMMUNITY_MEMBER_2
			));

			assert_eq!(Communities::members_count(COMMUNITY), Some(3));
			assert_eq!(Communities::membership(COMMUNITY, COMMUNITY_MEMBER_1), Some(()));
			assert_eq!(Communities::membership(COMMUNITY, COMMUNITY_MEMBER_2), Some(()));
		});
	}

	#[test]
	fn cannot_add_member_twice() {
		new_test_ext().execute_with(|| {
			setup();
			assert_ok!(Communities::do_force_complete_challenge(&COMMUNITY));

			// Successfully adds a member
			assert_ok!(Communities::add_member(
				RuntimeOrigin::signed(COMMUNITY_ADMIN),
				COMMUNITY,
				COMMUNITY_MEMBER_1
			));

			// Fails to add a member twice
			assert_noop!(
				Communities::add_member(RuntimeOrigin::signed(COMMUNITY_ADMIN), COMMUNITY, COMMUNITY_MEMBER_1),
				Error::AlreadyAMember
			);
		});
	}
}

mod remove_member {
	use super::*;

	#[test]
	fn fails_when_community_is_not_active() {
		new_test_ext().execute_with(|| {
			setup();

			assert_noop!(
				Communities::remove_member(RuntimeOrigin::signed(COMMUNITY_ADMIN), COMMUNITY, COMMUNITY_MEMBER_1),
				Error::CommunityNotActive
			);
		});
	}

	#[test]
	fn fails_when_caller_not_a_privleged_origin() {
		new_test_ext().execute_with(|| {
			setup();
			assert_ok!(Communities::do_force_complete_challenge(&COMMUNITY));
			assert_ok!(Communities::do_insert_member(&COMMUNITY, &COMMUNITY_MEMBER_1));

			assert_noop!(
				Communities::remove_member(RuntimeOrigin::none(), COMMUNITY, COMMUNITY_MEMBER_1),
				DispatchError::BadOrigin
			);

			assert_noop!(
				Communities::remove_member(RuntimeOrigin::signed(COMMUNITY_MEMBER_1), COMMUNITY, COMMUNITY_MEMBER_2),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn fails_when_not_a_community_member() {
		new_test_ext().execute_with(|| {
			setup();
			assert_ok!(Communities::do_force_complete_challenge(&COMMUNITY));
			assert_ok!(Communities::do_insert_member(&COMMUNITY, &COMMUNITY_MEMBER_1));

			assert_noop!(
				Communities::remove_member(RuntimeOrigin::signed(COMMUNITY_ADMIN), COMMUNITY, COMMUNITY_MEMBER_2),
				Error::NotAMember
			);

			assert_noop!(
				Communities::remove_member(RuntimeOrigin::root(), COMMUNITY, COMMUNITY_NON_MEMBER),
				Error::NotAMember
			);
		});
	}

	#[test]
	fn cannot_remove_admin() {
		new_test_ext().execute_with(|| {
			setup();
			assert_ok!(Communities::do_force_complete_challenge(&COMMUNITY));
			assert_ok!(Communities::do_insert_member(&COMMUNITY, &COMMUNITY_MEMBER_1));

			assert_noop!(
				Communities::remove_member(RuntimeOrigin::root(), COMMUNITY, COMMUNITY_ADMIN),
				Error::CannotRemoveAdmin
			);
		});
	}

	#[test]
	fn it_works() {
		new_test_ext().execute_with(|| {
			setup();
			assert_ok!(Communities::do_force_complete_challenge(&COMMUNITY));
			assert_ok!(Communities::do_insert_member(&COMMUNITY, &COMMUNITY_MEMBER_1));

			assert_ok!(Communities::remove_member(
				RuntimeOrigin::signed(COMMUNITY_ADMIN),
				COMMUNITY,
				COMMUNITY_MEMBER_1
			));

			assert_eq!(Communities::members_count(COMMUNITY), Some(1));
			assert_eq!(Communities::membership(COMMUNITY, COMMUNITY_MEMBER_1), None);
		});

		new_test_ext().execute_with(|| {
			setup();
			assert_ok!(Communities::do_force_complete_challenge(&COMMUNITY));
			assert_ok!(Communities::do_insert_member(&COMMUNITY, &COMMUNITY_MEMBER_1));

			assert_ok!(Communities::remove_member(
				RuntimeOrigin::root(),
				COMMUNITY,
				COMMUNITY_MEMBER_1
			));

			assert_eq!(Communities::members_count(COMMUNITY), Some(1));
			assert_eq!(Communities::membership(COMMUNITY, COMMUNITY_MEMBER_1), None);
		});
	}
}

mod member_rank {
	use super::*;

	fn setup() {
		super::setup();
		assert_ok!(Communities::do_force_complete_challenge(&COMMUNITY));

		assert_ok!(Communities::add_member(
			RuntimeOrigin::signed(COMMUNITY_ADMIN),
			COMMUNITY,
			COMMUNITY_MEMBER_1
		));
	}

	mod promote_member {
		use crate::MemberRanks;

		use super::*;

		#[test]
		fn fails_when_caller_not_a_privleged_origin() {
			new_test_ext().execute_with(|| {
				setup();

				assert_noop!(
					Communities::promote_member(
						RuntimeOrigin::signed(COMMUNITY_MEMBER_1),
						COMMUNITY,
						COMMUNITY_MEMBER_1
					),
					BadOrigin
				);
			});
		}

		#[test]
		fn fails_when_not_a_community_member() {
			new_test_ext().execute_with(|| {
				setup();

				assert_noop!(
					Communities::promote_member(
						RuntimeOrigin::signed(COMMUNITY_ADMIN),
						COMMUNITY,
						COMMUNITY_NON_MEMBER
					),
					Error::NotAMember,
				);
			});
		}

		#[test]
		fn it_works() {
			new_test_ext().execute_with(|| {
				setup();

				assert_ok!(Communities::promote_member(
					RuntimeOrigin::signed(COMMUNITY_ADMIN),
					COMMUNITY,
					COMMUNITY_MEMBER_1
				));
				assert_eq!(Communities::member_rank(COMMUNITY, COMMUNITY_MEMBER_1), 1.into());
			});
		}

		#[test]
		fn should_stay_at_max_rank() {
			new_test_ext().execute_with(|| {
				setup();

				MemberRanks::<Test>::set(COMMUNITY, COMMUNITY_MEMBER_1, Rank::MAX);
				assert_ok!(Communities::promote_member(
					RuntimeOrigin::signed(COMMUNITY_ADMIN),
					COMMUNITY,
					COMMUNITY_MEMBER_1
				));

				assert_eq!(Communities::member_rank(COMMUNITY, COMMUNITY_MEMBER_1), Rank::MAX);
			});
		}
	}

	mod demote_member {
		use crate::MemberRanks;

		use super::*;

		#[test]
		fn fails_when_caller_not_a_privleged_origin() {
			new_test_ext().execute_with(|| {
				setup();

				assert_noop!(
					Communities::demote_member(
						RuntimeOrigin::signed(COMMUNITY_MEMBER_1),
						COMMUNITY,
						COMMUNITY_MEMBER_1
					),
					BadOrigin
				);
			});
		}

		#[test]
		fn fails_when_not_a_community_member() {
			new_test_ext().execute_with(|| {
				setup();

				assert_noop!(
					Communities::demote_member(RuntimeOrigin::signed(COMMUNITY_ADMIN), COMMUNITY, COMMUNITY_NON_MEMBER),
					Error::NotAMember,
				);
			});
		}

		#[test]
		fn it_works() {
			new_test_ext().execute_with(|| {
				setup();

				MemberRanks::<Test>::set(COMMUNITY, COMMUNITY_MEMBER_1, 2.into());

				assert_ok!(Communities::demote_member(
					RuntimeOrigin::signed(COMMUNITY_ADMIN),
					COMMUNITY,
					COMMUNITY_MEMBER_1
				));
				assert_eq!(Communities::member_rank(COMMUNITY, COMMUNITY_MEMBER_1), 1.into());
			});
		}

		#[test]
		fn should_remain_at_min_rank() {
			new_test_ext().execute_with(|| {
				setup();

				MemberRanks::<Test>::set(COMMUNITY, COMMUNITY_MEMBER_1, 0.into());
				assert_ok!(Communities::demote_member(
					RuntimeOrigin::signed(COMMUNITY_ADMIN),
					COMMUNITY,
					COMMUNITY_MEMBER_1
				));
				assert_eq!(Communities::member_rank(COMMUNITY, COMMUNITY_MEMBER_1), Rank::MIN);
			});
		}
	}
}
