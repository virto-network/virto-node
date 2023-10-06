use super::*;
use frame_support::traits::{
	fungible::{Inspect as FunInspect, Unbalanced},
	fungibles::{Create, Inspect, Mutate},
	tokens::{Fortitude::Polite, Preservation::Preserve},
};
use sp_runtime::TokenError;

const ALICE: AccountId = 40;
const BOB: AccountId = 41;
const COMMUNITY_MEMBER_1: AccountId = 43;

const ASSET_A: AssetId = 100;

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
