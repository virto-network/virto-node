use crate::types::*;
use crate::{mock::*, CommunityInfo, Error as PalletError};
use frame_support::{assert_noop, assert_ok};

type Error = PalletError<Test>;

const COMMUNITY: u128 = 1;
const COMMUNITY_ADMIN: u64 = 42;

mod apply {
	use super::*;
	use frame_support::traits::fungible;

	mod do_register_community {
		use super::*;

		#[test]
		fn fails_if_community_already_exists() {
			new_test_ext().execute_with(|| {
				// Emulate a preexisting community
				<CommunityInfo<Test>>::insert(
					COMMUNITY,
					Community {
						admin: COMMUNITY_ADMIN,
						state: CommunityState::Awaiting,
						sufficient_asset_id: None,
					},
				);
				assert_ok!(Communities::do_insert_member(&COMMUNITY, &COMMUNITY_ADMIN));

				// Should fail adding the community
				assert_noop!(
					Communities::do_register_community(&COMMUNITY_ADMIN, &COMMUNITY),
					Error::CommunityAlreadyExists
				);

				// Assert that the correct event was deposited
				// System::assert_last_event(Event::SomethingStored { something:
				// 42, who: 1 }.into());
			});
		}

		#[test]
		fn it_works() {
			new_test_ext().execute_with(|| {
				assert_ok!(Communities::do_register_community(&1, &42));
			});
		}
	}

	mod do_create_community_account {
		use frame_support::traits::fungible::InspectFreeze;

		use super::*;

		#[test]
		fn fails_if_not_enough_funds_to_take_deposit() {
			new_test_ext().execute_with(|| {
				assert_noop!(
					Communities::do_create_community_account(&COMMUNITY_ADMIN, &COMMUNITY),
					sp_runtime::DispatchError::Arithmetic(sp_runtime::ArithmeticError::Underflow)
				);
			});
		}

		#[test]
		fn it_works() {
			new_test_ext().execute_with(|| {
				let minimum_balance = <<Test as crate::Config>::Balances as fungible::Inspect<
					<Test as frame_system::Config>::AccountId,
				>>::minimum_balance();

				assert_ok!(Balances::force_set_balance(
					RuntimeOrigin::root(),
					COMMUNITY_ADMIN,
					2 * minimum_balance,
				));

				assert_ok!(Communities::do_create_community_account(&COMMUNITY_ADMIN, &COMMUNITY));

				assert_eq!(
					Balances::balance_frozen(&(), &Communities::get_community_account_id(&COMMUNITY)),
					minimum_balance
				);
				assert_eq!(
					Balances::usable_balance(Communities::get_community_account_id(&COMMUNITY)),
					0
				);
			});
		}
	}

	mod call {
		use crate::Event;

		use super::*;

		#[test]
		fn it_works() {
			new_test_ext().execute_with(|| {
				System::set_block_number(1);

				let minimum_balance = <<Test as crate::Config>::Balances as fungible::Inspect<
					<Test as frame_system::Config>::AccountId,
				>>::minimum_balance();

				assert_ok!(Balances::force_set_balance(
					RuntimeOrigin::root(),
					COMMUNITY_ADMIN,
					2 * minimum_balance,
				));

				assert_ok!(Communities::apply(RuntimeOrigin::signed(COMMUNITY_ADMIN), COMMUNITY));

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
}
