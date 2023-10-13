use super::*;
use crate::{tests::helpers::run_to_block, Event};
use sp_core::{blake2_256, H256};

fn setup() {
	super::setup();
	assert_ok!(Communities::do_force_complete_challenge(&COMMUNITY));
}

fn call_remark() -> RuntimeCall {
	frame_system::Call::remark_with_event {
		remark: b"Hello, governance!".to_vec(),
	}
	.into()
}

mod open_proposal {
	use super::*;

	#[test]
	fn fails_if_not_called_by_a_community_member() {
		new_test_ext().execute_with(|| {
			setup();

			assert_noop!(
				Communities::open_proposal(RuntimeOrigin::none(), COMMUNITY, Box::new(call_remark())),
				DispatchError::BadOrigin
			);

			assert_noop!(
				Communities::open_proposal(RuntimeOrigin::root(), COMMUNITY, Box::new(call_remark())),
				DispatchError::BadOrigin
			);
		})
	}

	#[test]
	fn it_works() {
		new_test_ext().execute_with(|| {
			setup();

			assert_ok!(Communities::open_proposal(
				RuntimeOrigin::signed(COMMUNITY_ADMIN),
				COMMUNITY,
				Box::new(call_remark())
			));

			run_to_block(3);

			assert!(System::events().iter().any(|record| {
				record.event
					== Event::<Test>::ProposalEnqueued {
						community_id: COMMUNITY,
						proposer: COMMUNITY_ADMIN,
					}
					.into()
			}));

			assert!(Communities::proposals(COMMUNITY).iter().len() == 1);

			Communities::proposals(COMMUNITY).iter().for_each(|p| {
				println!("{:#?}", &p);
			});
		});
	}
}

mod execute_call {
	use super::*;

	const COMMUNITY_MEMBER_1: AccountId = 43;

	fn setup() {
		super::setup();
		assert_ok!(Communities::add_member(
			RuntimeOrigin::signed(COMMUNITY_ADMIN),
			COMMUNITY,
			COMMUNITY_MEMBER_1
		));
	}

	#[test]
	fn fails_if_not_called_by_a_community_admin() {
		new_test_ext().execute_with(|| {
			setup();

			assert_noop!(
				Communities::execute_call(RuntimeOrigin::none(), COMMUNITY, Box::new(call_remark())),
				DispatchError::BadOrigin
			);

			assert_noop!(
				Communities::execute_call(RuntimeOrigin::root(), COMMUNITY, Box::new(call_remark())),
				DispatchError::BadOrigin
			);

			assert_noop!(
				Communities::execute_call(
					RuntimeOrigin::signed(COMMUNITY_MEMBER_1),
					COMMUNITY,
					Box::new(call_remark())
				),
				DispatchError::BadOrigin
			);
		})
	}

	#[test]
	fn it_works() {
		new_test_ext().execute_with(|| {
			setup();

			assert_ok!(Communities::execute_call(
				RuntimeOrigin::signed(COMMUNITY_ADMIN),
				COMMUNITY,
				Box::new(call_remark())
			));

			println!("Events for block 1");
			System::events().iter().for_each(|record| {
				println!("{:#?}", &record.event);
			});

			assert!(System::events().iter().any(|record| {
				record.event
					== Event::<Test>::ProposalEnqueued {
						community_id: COMMUNITY,
						proposer: COMMUNITY_ADMIN,
					}
					.into()
			}));

			run_to_block(2);

			println!("Events for block 2");
			System::events().iter().for_each(|record| {
				println!("{:#?}", &record.event);
			});

			run_to_block(3);

			println!("Events for block 3");
			System::events().iter().for_each(|record| {
				println!("{:#?}", &record.event);
			});

			let community_account_id = Communities::get_community_account_id(&COMMUNITY);
			assert!(System::events().iter().any(|record| {
				println!("{:#?}", &record.event);
				record.event
					== frame_system::Event::<Test>::Remarked {
						sender: community_account_id,
						hash: H256::from(blake2_256(&b"Hello, governance!".to_vec())),
					}
					.into()
			}));
		});
	}
}
