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
