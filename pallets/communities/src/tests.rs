use crate::types::*;
use crate::{mock::*, CommunityInfo, Error as PalletError};
use frame_support::traits::fungible;
use frame_support::{assert_noop, assert_ok};

type Error = PalletError<Test>;

const COMMUNITY: u128 = 1;
const COMMUNITY_ADMIN: u64 = 42;

mod apply {
	use super::*;

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
					<Balances as fungible::InspectFreeze<<Test as frame_system::Config>::AccountId>>::balance_frozen(
						&(),
						&Communities::get_community_account_id(&COMMUNITY)
					),
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

mod set_metadata {
	use super::*;
	use sp_runtime::BoundedVec;

	fn setup() {
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
	}

	#[test]
	fn fails_if_bad_origin() {
		new_test_ext().execute_with(|| {
			setup();

			// Fail if trying to call from unsigned origin
			assert_noop!(
				Communities::set_metadata(RuntimeOrigin::none(), COMMUNITY, None, None, None, None),
				sp_runtime::DispatchError::BadOrigin
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
				sp_runtime::DispatchError::BadOrigin
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

			assert!(<crate::CommunityMetadata<Test>>::contains_key(COMMUNITY));
			let community_metadata = <crate::CommunityMetadata<Test>>::get(COMMUNITY)
				.expect("We've already asserted that the key is contained; qed");

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

			assert!(<crate::CommunityMetadata<Test>>::contains_key(COMMUNITY));
			let community_metadata = <crate::CommunityMetadata<Test>>::get(COMMUNITY)
				.expect("We've already asserted that the key is contained; qed");

			assert_eq!(
				community_metadata,
				crate::types::CommunityMetadata {
					name: BoundedVec::truncate_from(b"Virto Network".to_vec()),
					description: BoundedVec::truncate_from(b"A community of awesome builders".to_vec()),
					urls: BoundedVec::new(),
					locations: BoundedVec::new()
				}
			);
		});
	}
}
