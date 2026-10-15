//! `pallet_migrations` as Kreivo wires it (`DEC-30`, `F-12`).
//!
//! The stepped migration these tests run is a no-op that costs most of `MaxServiceWeight` per
//! step (`migrations::test_migration`): it is only registered in test builds, and the benchmark
//! build swaps it for the pallet's mocked migrations, so the tests that run it are compiled out
//! under `runtime-benchmarks`.

use super::*;

use crate::{config::system::MigrationsMaxServiceWeight, MultiBlockMigrations};
use frame_support::{
	traits::{Get, Hooks},
	weights::Weight,
};

// NFR-9, DEC-30: the deployment document's values (§1).
#[test]
fn the_service_weight_and_the_limits_are_the_deployment_values() {
	assert_eq!(
		MigrationsMaxServiceWeight::get(),
		Weight::from_parts(533_333_333_333, 2_796_202)
	);
	assert_eq!(
		<<Runtime as pallet_migrations::Config>::CursorMaxLen as Get<u32>>::get(),
		65_536
	);
	assert_eq!(
		<<Runtime as pallet_migrations::Config>::IdentifierMaxLen as Get<u32>>::get(),
		256
	);
}

// DEC-30, RISK-14: the service weight leaves room in the block for inherents and `on_initialize`.
#[test]
fn the_service_weight_fits_in_a_block_and_the_pallet_integrity_test_passes() {
	TestExternalities::default().execute_with(|| {
		let max_block = <Runtime as frame_system::Config>::BlockWeights::get().max_block;
		assert!(MigrationsMaxServiceWeight::get().all_lte(max_block));
		assert!(MigrationsMaxServiceWeight::get().all_lt(max_block));

		<MultiBlockMigrations as Hooks<_>>::integrity_test();
	});
}

#[cfg(not(feature = "runtime-benchmarks"))]
mod stepped {
	use super::*;

	use crate::{migrations::test_migration::*, RuntimeEvent, System};
	use frame_support::{migrations::MultiStepMigrator, traits::OnRuntimeUpgrade};
	use pallet_migrations::{Cursor, Event as MigrationsEvent, MbmIsOngoing, MigrationCursor};

	/// A block's worth of migration: the executive steps the migrator once after the inherents.
	fn next_block() -> Weight {
		System::set_block_number(System::block_number() + 1);
		<Runtime as frame_system::Config>::MultiBlockMigrator::step()
	}

	fn the_upgrade_block() {
		System::set_block_number(1);
		<MultiBlockMigrations as OnRuntimeUpgrade>::on_runtime_upgrade();
	}

	// NFR-9, DEC-30: a stepped migration takes one step a block, each under `MaxServiceWeight`, and
	// the block builder takes only inherents (the executive's `OnlyInherents` mode, which is
	// `ongoing()`) until it is done; user transactions wait in the pool and go in after.
	#[test]
	fn a_stepped_migration_spans_several_blocks_and_then_lets_transactions_in() {
		TestExternalities::default().execute_with(|| {
			NoopSteps::set(&4);
			the_upgrade_block();

			assert!(<Runtime as frame_system::Config>::MultiBlockMigrator::ongoing());
			assert_eq!(MultiBlockMigrations::ongoing_status(), MbmIsOngoing::Yes);

			for _ in 0..3 {
				let used = next_block();
				assert!(used.all_gte(NoopStepped::step_weight()));
				assert!(used.all_lte(MigrationsMaxServiceWeight::get()));
				assert!(
					<Runtime as frame_system::Config>::MultiBlockMigrator::ongoing(),
					"transactions are still held"
				);
			}

			next_block();
			assert!(!<Runtime as frame_system::Config>::MultiBlockMigrator::ongoing());
			assert!(Cursor::<Runtime>::get().is_none());
			assert!(System::events().iter().any(|e| matches!(
				e.event,
				RuntimeEvent::MultiBlockMigrations(MigrationsEvent::UpgradeCompleted)
			)));

			// Done: nothing runs again.
			assert!(next_block().all_lt(NoopStepped::step_weight()));
		});
	}

	// NFR-9, DEC-30: a migration that has run is recorded, and a later upgrade doesn't run it again.
	#[test]
	fn a_finished_migration_does_not_run_again_on_a_later_upgrade() {
		TestExternalities::default().execute_with(|| {
			NoopSteps::set(&1);
			the_upgrade_block();
			next_block();
			assert!(!<Runtime as frame_system::Config>::MultiBlockMigrator::ongoing());

			// The upgrade onboards the list again; the next step skips what the history holds.
			<MultiBlockMigrations as OnRuntimeUpgrade>::on_runtime_upgrade();
			assert!(next_block().all_lt(NoopStepped::step_weight()));
			assert!(!<Runtime as frame_system::Config>::MultiBlockMigrator::ongoing());
		});
	}

	// DEC-30, RISK-14: a failing migration freezes the chain: it stays stuck, and transactions are
	// never let in, until governance clears the cursor.
	#[test]
	fn a_failing_migration_freezes_the_chain() {
		TestExternalities::default().execute_with(|| {
			NoopSteps::set(&4);
			NoopFailsAt::set(&Some(1));
			the_upgrade_block();

			next_block();
			assert_eq!(MultiBlockMigrations::ongoing_status(), MbmIsOngoing::Yes);
			next_block();

			assert_eq!(Cursor::<Runtime>::get(), Some(MigrationCursor::Stuck));
			assert_eq!(MultiBlockMigrations::ongoing_status(), MbmIsOngoing::Stuck);
			assert!(<Runtime as frame_system::Config>::MultiBlockMigrator::ongoing());
			assert!(System::events().iter().any(|e| matches!(
				e.event,
				RuntimeEvent::MultiBlockMigrations(MigrationsEvent::MigrationFailed { .. })
			)));

			// It stays frozen however many blocks go by.
			for _ in 0..5 {
				next_block();
				assert_eq!(Cursor::<Runtime>::get(), Some(MigrationCursor::Stuck));
				assert!(<Runtime as frame_system::Config>::MultiBlockMigrator::ongoing());
			}
		});
	}

	// DEC-30: root clears a stuck migration, and the chain takes transactions again.
	#[test]
	fn root_can_clear_a_stuck_migration() {
		TestExternalities::default().execute_with(|| {
			NoopFailsAt::set(&Some(0));
			the_upgrade_block();
			next_block();
			assert_eq!(MultiBlockMigrations::ongoing_status(), MbmIsOngoing::Stuck);

			frame_support::assert_ok!(MultiBlockMigrations::force_set_cursor(RuntimeOrigin::root(), None));
			assert!(!<Runtime as frame_system::Config>::MultiBlockMigrator::ongoing());
		});
	}
}
