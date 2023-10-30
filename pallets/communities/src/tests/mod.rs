use crate::types::{self, *};
use frame_support::{assert_noop, assert_ok, traits::fungible};
use sp_runtime::{ArithmeticError, DispatchError};

mod mock;
pub use mock::*;

mod helpers;

type Error = crate::Error<Test>;

const COMMUNITY: CommunityId = 1;
const COMMUNITY_ADMIN: AccountId = 42;

mod governance;
mod membership;
mod registry;
mod treasury;

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
