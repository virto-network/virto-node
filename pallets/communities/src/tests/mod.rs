use frame_support::assert_ok;

mod helpers;

use crate::mock::*;
use helpers::*;

type Error = crate::Error<Test>;

mod governance;
mod membership;
mod registry;
mod weights;
