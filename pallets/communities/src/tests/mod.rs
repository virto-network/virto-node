use crate::types::{self, *};
use frame_support::{assert_noop, assert_ok, traits::fungible};
use sp_runtime::traits::BadOrigin;
use sp_runtime::{ArithmeticError, DispatchError};

mod mock;
use mock::*;

type Error = crate::Error<Test>;

const COMMUNITY: CommunityId = 1;
const COMMUNITY_ADMIN: AccountId = 42;

mod membership;
mod registry;

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
	assert_ok!(Communities::apply_for(
		RuntimeOrigin::signed(COMMUNITY_ADMIN),
		COMMUNITY
	));
}

fn activate_community(entity_id: CommunityId) {
	assert_ok!(Challenger::<Test>::register(entity_id.clone()));
	assert_ok!(Challenger::<Test>::validate(entity_id.clone(), true));
}
