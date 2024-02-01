use frame_support::assert_ok;

mod helpers;
mod mock;

use helpers::*;
use mock::*;

type Error = crate::Error<Test>;

mod governance;
mod membership;
mod registry;
