use frame_support::assert_ok;

mod helpers;
mod mock;

use helpers::next_block;
use mock::*;

type Error = crate::Error<Test>;

mod governance;
mod membership;
mod registry;
