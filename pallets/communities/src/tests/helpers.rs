use super::*;
use frame_support::traits::Hooks;
use frame_system::pallet_prelude::BlockNumberFor;

pub fn tick_block() {
	on_finalize();

	// if System::block_number() > 1 {
	// 	println!(
	// 		"Finished block {} with events:\n{}\n\n",
	// 		System::block_number(),
	// 		&System::events()
	// 			.into_iter()
	// 			.map(|ev| format!("\t{:?}", &ev))
	// 			.collect::<Vec<_>>()
	// 			.join("\n")
	// 	);
	// }
	System::reset_events();
	System::set_block_number(System::block_number() + 1);
	// println!("Starting block {}", System::block_number());

	on_initialize();
}

pub fn tick_blocks(n: BlockNumberFor<Test>) {
	for _ in 0..n {
		tick_block();
	}
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
