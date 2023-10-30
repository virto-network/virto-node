use super::*;
use frame_support::traits::{OnFinalize, OnInitialize};

pub(super) fn run_to_block(n: u64) {
	let current_block = System::block_number();
	assert!(n > current_block);

	while System::block_number() < n {
		Assets::on_finalize(System::block_number());
		Balances::on_finalize(System::block_number());
		Communities::on_finalize(System::block_number());
		Preimage::on_finalize(System::block_number());
		Scheduler::on_finalize(System::block_number());

		System::reset_events();
		System::set_block_number(System::block_number() + 1);

		System::on_initialize(System::block_number());
		Assets::on_initialize(System::block_number());
		Balances::on_initialize(System::block_number());
		Preimage::on_initialize(System::block_number());
		Scheduler::on_initialize(System::block_number());
	}
}
