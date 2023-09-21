use crate::types::*;
use crate::{mock::*, CommunityInfo, Error as PalletError};
use frame_support::traits::fungible;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{ArithmeticError, DispatchError};

mod helpers;
use helpers::*;

pub(self) type CommunityId = CommunityIdOf<Test>;
type Error = PalletError<Test>;

const COMMUNITY: CommunityId = 1;
const COMMUNITY_ADMIN: AccountId = 42;

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

mod registry {
	use super::*;

	mod apply {
		use super::*;
		use crate::Event;

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
		use sp_runtime::BoundedVec;

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
}

mod membership {
	use super::*;

	mod add_member {
		use super::*;
		use crate::{mock::new_test_ext, tests::COMMUNITY, CommunityMembers, CommunityMembersCount};

		const COMMUNITY_MEMBER_1: u64 = 43;
		const COMMUNITY_MEMBER_2: u64 = 44;
		const COMMUNITY_NON_MEMBER: u64 = 45;

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

				assert_eq!(<CommunityMembersCount<Test>>::get(COMMUNITY), Some(3));
				assert_eq!(<CommunityMembers<Test>>::get(COMMUNITY, COMMUNITY_MEMBER_1), Some(()));
				assert_eq!(<CommunityMembers<Test>>::get(COMMUNITY, COMMUNITY_MEMBER_2), Some(()));
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
		use crate::{mock::new_test_ext, tests::COMMUNITY, CommunityMembers, CommunityMembersCount};

		const COMMUNITY_MEMBER_1: u64 = 43;
		const COMMUNITY_MEMBER_2: u64 = 44;
		const COMMUNITY_NON_MEMBER: u64 = 45;

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
					Communities::remove_member(
						RuntimeOrigin::signed(COMMUNITY_MEMBER_1),
						COMMUNITY,
						COMMUNITY_MEMBER_2
					),
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

				assert_eq!(<CommunityMembersCount<Test>>::get(COMMUNITY), Some(1));
				assert_eq!(<CommunityMembers<Test>>::get(COMMUNITY, COMMUNITY_MEMBER_1), None);
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

				assert_eq!(<CommunityMembersCount<Test>>::get(COMMUNITY), Some(1));
				assert_eq!(<CommunityMembers<Test>>::get(COMMUNITY, COMMUNITY_MEMBER_1), None);
			});
		}
	}
}

mod treasury {
	use super::*;
	use frame_support::traits::{
		fungible::{Inspect as FunInspect, Unbalanced},
		fungibles::{Create, Inspect, Mutate},
		tokens::{Fortitude::Polite, Preservation::Preserve},
	};
	use sp_runtime::TokenError;

	const ALICE: u64 = 40;
	const BOB: u64 = 41;
	const COMMUNITY_MEMBER_1: u64 = 43;

	const ASSET_A: u32 = 100;

	fn setup() {
		super::setup();

		// Let's activate the community
		assert_ok!(Communities::do_force_complete_challenge(&COMMUNITY));
		let community_account_id = Communities::get_community_account_id(&COMMUNITY);

		// Let's mint some balance
		assert_ok!(Balances::increase_balance(
			&ALICE,
			1,
			frame_support::traits::tokens::Precision::Exact
		));

		// Let's issue/mint some assets
		let minimum_balance = 1;

		assert_ok!(<Assets as Create<AccountIdOf<Test>>>::create(
			ASSET_A,
			community_account_id,
			true,
			minimum_balance
		));

		assert_ok!(<Assets as Mutate<AccountIdOf<Test>>>::mint_into(
			ASSET_A,
			&ALICE,
			minimum_balance
				.checked_add(1)
				.expect("This should not overflow as ED is way below U128::MAX; qed")
		));
		assert_ok!(<Assets as Mutate<AccountIdOf<Test>>>::mint_into(
			ASSET_A,
			&community_account_id,
			minimum_balance
		));

		// Let's add COMMUNITY_MEMBER_1 to the community
		assert_ok!(Communities::do_insert_member(&COMMUNITY, &COMMUNITY_MEMBER_1));
	}

	mod assets_transfer {
		use super::*;

		#[test]
		fn fails_if_bad_origin() {
			new_test_ext().execute_with(|| {
				setup();

				// Fail if trying to call from unsigned origin
				assert_noop!(
					Communities::assets_transfer(RuntimeOrigin::none(), COMMUNITY, ASSET_A, BOB, 1),
					DispatchError::BadOrigin
				);

				// Fail if trying to call from non-admin
				assert_noop!(
					Communities::assets_transfer(RuntimeOrigin::signed(COMMUNITY_MEMBER_1), COMMUNITY, ASSET_A, BOB, 1),
					DispatchError::BadOrigin
				);
			});
		}

		#[test]
		fn fails_if_not_enough_balance() {
			new_test_ext().execute_with(|| {
				setup();

				assert_noop!(
					Communities::assets_transfer(RuntimeOrigin::signed(COMMUNITY_ADMIN), COMMUNITY, ASSET_A, BOB, 1),
					TokenError::NotExpendable,
				);
			});
		}

		#[test]
		fn it_works() {
			new_test_ext().execute_with(|| {
				setup();
				let community_account_id = Communities::get_community_account_id(&COMMUNITY);

				assert_ok!(Assets::transfer(
					RuntimeOrigin::signed(ALICE),
					codec::Compact(ASSET_A),
					community_account_id,
					1
				));

				assert_ok!(Communities::assets_transfer(
					RuntimeOrigin::signed(COMMUNITY_ADMIN),
					COMMUNITY,
					ASSET_A,
					BOB,
					1
				));

				assert_eq!(Assets::reducible_balance(ASSET_A, &ALICE, Preserve, Polite), 0);
				assert_eq!(
					Assets::reducible_balance(ASSET_A, &community_account_id, Preserve, Polite),
					0
				);
				assert_eq!(Assets::reducible_balance(ASSET_A, &BOB, Preserve, Polite), 0);
			});
		}
	}

	mod balances_transfer {
		use super::*;

		#[test]
		fn fails_if_bad_origin() {
			new_test_ext().execute_with(|| {
				setup();

				// Fail if trying to call from unsigned origin
				assert_noop!(
					Communities::balance_transfer(RuntimeOrigin::none(), COMMUNITY, BOB, 1),
					DispatchError::BadOrigin
				);

				// Fail if trying to call from non-admin
				assert_noop!(
					Communities::balance_transfer(RuntimeOrigin::signed(COMMUNITY_MEMBER_1), COMMUNITY, BOB, 1),
					DispatchError::BadOrigin
				);
			});
		}

		#[test]
		fn fails_if_not_enough_balance() {
			new_test_ext().execute_with(|| {
				setup();

				assert_noop!(
					Communities::balance_transfer(RuntimeOrigin::signed(COMMUNITY_ADMIN), COMMUNITY, BOB, 1),
					TokenError::Frozen,
				);
			});
		}

		#[test]
		fn it_works() {
			new_test_ext().execute_with(|| {
				setup();
				let community_account_id = Communities::get_community_account_id(&COMMUNITY);

				assert_ok!(Balances::transfer(
					RuntimeOrigin::signed(ALICE),
					community_account_id,
					1
				));

				assert_ok!(Communities::balance_transfer(
					RuntimeOrigin::signed(COMMUNITY_ADMIN),
					COMMUNITY,
					BOB,
					1
				));

				assert_eq!(Balances::reducible_balance(&ALICE, Preserve, Polite), 0);
				assert_eq!(Balances::reducible_balance(&community_account_id, Preserve, Polite), 0);
				assert_eq!(Balances::reducible_balance(&BOB, Preserve, Polite), 0);
			});
		}
	}
}

mod fungibles {
	use super::*;
	use frame_support::traits::{fungible::Unbalanced, fungibles::Create, tokens::Precision::Exact, Currency};

	const COMMUNITY_B: CommunityId = 2;

	const ALICE: AccountId = 40;
	const BOB: AccountId = 41;
	const COMMUNITY_MEMBER_1: AccountId = 43;

	const ASSET_A: AssetId = 100;
	const ASSET_B: AssetId = 101;
	const ASSET_C: AssetId = 102;
	const ASSET_D: AssetId = 103;

	fn setup() {
		super::setup();

		assert_ok!(<Assets as Create<AccountIdOf<Test>>>::create(ASSET_A, ALICE, true, 1));

		let minimum_balance = Balances::minimum_balance();
		assert_ok!(Balances::increase_balance(&BOB, 2 * minimum_balance, Exact));
		assert_ok!(Communities::apply(RuntimeOrigin::signed(BOB), COMMUNITY_B));

		assert_ok!(Communities::do_force_complete_challenge(&COMMUNITY));
		assert_ok!(Communities::do_force_complete_challenge(&COMMUNITY_B));
	}

	mod create_asset {
		use super::*;

		#[test]
		fn fails_if_bad_origin() {
			new_test_ext().execute_with(|| {
				setup();

				// Fail if trying to call from unsigned origin
				assert_noop!(
					Communities::create_asset(RuntimeOrigin::none(), COMMUNITY, ASSET_A, 1),
					DispatchError::BadOrigin
				);

				// Fail if trying to call from non-admin
				assert_noop!(
					Communities::create_asset(RuntimeOrigin::signed(COMMUNITY_MEMBER_1), COMMUNITY, ASSET_A, 1),
					DispatchError::BadOrigin
				);
			});
		}

		#[test]
		fn fails_if_asset_exists() {
			new_test_ext().execute_with(|| {
				setup();

				assert_noop!(
					Communities::create_asset(RuntimeOrigin::signed(COMMUNITY_ADMIN), COMMUNITY, ASSET_A, 1),
					pallet_assets::Error::<Test>::InUse
				);
			});
		}

		#[test]
		fn it_works() {
			new_test_ext().execute_with(|| {
				setup();

				// Can register a new asset
				assert_ok!(Communities::create_asset(
					RuntimeOrigin::signed(COMMUNITY_ADMIN),
					COMMUNITY,
					ASSET_B,
					1
				));

				// Can register additional assets
				assert_ok!(Communities::create_asset(
					RuntimeOrigin::signed(COMMUNITY_ADMIN),
					COMMUNITY,
					ASSET_C,
					1
				));

				// First asset owned by the community is sufficient by default
				assert_sufficiency(COMMUNITY, ASSET_B, 1, true);

				// Additional assets owned by the community are not sufficient
				// by default
				assert_sufficiency(COMMUNITY, ASSET_C, 1, false);
			});
		}
	}

	mod destroy_asset {
		use super::*;

		fn setup() {
			super::setup();

			assert_ok!(Communities::create_asset(
				RuntimeOrigin::signed(COMMUNITY_ADMIN),
				COMMUNITY,
				ASSET_B,
				1
			));
			assert_ok!(Communities::create_asset(
				RuntimeOrigin::signed(BOB),
				COMMUNITY_B,
				ASSET_C,
				1
			));
		}

		#[test]
		fn fails_if_bad_origin() {
			new_test_ext().execute_with(|| {
				setup();

				// Fail if trying to call from unsigned origin
				assert_noop!(
					Communities::destroy_asset(RuntimeOrigin::none(), COMMUNITY, ASSET_A),
					DispatchError::BadOrigin
				);

				// Fail if trying to call from non-admin
				assert_noop!(
					Communities::destroy_asset(RuntimeOrigin::signed(COMMUNITY_MEMBER_1), COMMUNITY, ASSET_A),
					DispatchError::BadOrigin
				);
			});
		}

		#[test]
		fn fails_if_asset_does_not_exist() {
			new_test_ext().execute_with(|| {
				setup();

				assert_noop!(
					Communities::destroy_asset(RuntimeOrigin::signed(COMMUNITY_ADMIN), COMMUNITY, ASSET_D),
					Error::UnknownAsset,
				);
			});
		}

		#[test]
		fn fails_if_asset_is_not_controlled_by_the_community() {
			new_test_ext().execute_with(|| {
				setup();

				assert_noop!(
					Communities::destroy_asset(RuntimeOrigin::signed(COMMUNITY_ADMIN), COMMUNITY, ASSET_C),
					Error::CannotDestroyUncontrolledAsset,
				);

				assert_noop!(
					Communities::destroy_asset(RuntimeOrigin::signed(BOB), COMMUNITY_B, ASSET_B),
					Error::CannotDestroyUncontrolledAsset,
				);
			});
		}

		#[test]
		fn it_works() {
			new_test_ext().execute_with(|| {
				setup();

				assert_ok!(Communities::destroy_asset(
					RuntimeOrigin::signed(COMMUNITY_ADMIN),
					COMMUNITY,
					ASSET_B
				));
				assert!(get_asset(ASSET_B).is_none());

				assert_ok!(Communities::destroy_asset(
					RuntimeOrigin::signed(BOB),
					COMMUNITY_B,
					ASSET_C
				));
				assert!(get_asset(ASSET_B).is_none());
			});
		}
	}
}
