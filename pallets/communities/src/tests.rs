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

mod fungibles;
mod membership;
mod registry;
mod treasury;
