use frame_support::assert_ok;

mod mock;
use mock::*;

type Error = crate::Error<Test>;

mod membership;
mod registry;
