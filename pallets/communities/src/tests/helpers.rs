use super::*;
use frame_support::traits::Hooks;

pub fn next_block() {
	on_finalize();

	System::reset_events();
	System::set_block_number(System::block_number() + 1);

	on_initialize();
}

fn on_finalize() {
	let block_number = System::block_number();

	Referenda::on_finalize(block_number);
	Scheduler::on_finalize(block_number);
	Tracks::on_finalize(block_number);
	Communities::on_finalize(block_number);
	Nfts::on_finalize(block_number);
	Assets::on_finalize(block_number);
	Balances::on_finalize(block_number);
	System::on_finalize(block_number);
}

fn on_initialize() {
	let block_number = System::block_number();

	Referenda::on_initialize(block_number);
	Scheduler::on_initialize(block_number);
	Tracks::on_initialize(block_number);
	Communities::on_initialize(block_number);
	Nfts::on_initialize(block_number);
	Assets::on_initialize(block_number);
	Balances::on_initialize(block_number);
	System::on_initialize(block_number);
}
