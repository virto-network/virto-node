//! The listings benchmarks need Kreivo's origins and bindings to produce their setups: when one
//! cannot, the benchmark is skipped as weightless and its call is free.

use super::*;
use crate::Listings;

/// The setup the subscription benchmarks share (`subscribe`, `cancel_subscription`,
/// `terminate_subscription`, `schedule_replacement`, `amend_subscription`, `cancel_replacement`,
/// `process_due_replacement` and `settle_subscription`), run in the runtime: an inventory with a
/// subscription item, and a funded subscriber that subscribes to it.
#[cfg(feature = "runtime-benchmarks")]
#[test]
fn listings_benchmarks_run_in_the_runtime() {
	use frame_support::traits::{fungible::Unbalanced, EnsureOriginWithArg};
	use pallet_listings::{
		AmendmentPolicy, Config, ItemPriceOf, SubscriptionConditionsOf, SubscriptionPolicy,
		SubscriptionsBenchmarkHelper,
	};
	use sp_runtime::traits::BlockNumberProvider;

	type Instance = pallet_listings::Instance1;
	type Helper = <Runtime as Config<Instance>>::BenchmarkHelper;

	TestExternalities::default().execute_with(|| {
		crate::System::set_block_number(1);
		crate::config::RelaychainData::set_block_number(1);

		let inventory_id = <Helper as pallet_listings::BenchmarkHelper<_>>::inventory_id();

		// The inventory's admin, with funds for the catalog's deposits.
		let origin =
			<<Runtime as Config<Instance>>::CreateInventoryOrigin as EnsureOriginWithArg<_, _>>::try_successful_origin(
				&inventory_id,
			)
			.expect("the inventory origin is expected to be producible; qed");
		let owner = <<Runtime as Config<Instance>>::CreateInventoryOrigin as EnsureOriginWithArg<_, _>>::try_origin(
			origin.clone(),
			&inventory_id,
		)
		.ok()
		.expect("the inventory origin is expected to succeed; qed");
		assert_ok!(Balances::increase_balance(
			&owner,
			u128::MAX / 2,
			frame_support::traits::tokens::Precision::Exact
		));

		assert_ok!(Listings::create_inventory(origin.clone(), inventory_id));
		assert_ok!(Listings::publish_item(
			origin.clone(),
			inventory_id,
			0,
			b"subscription".to_vec().try_into().unwrap(),
			None
		));

		let (asset, amount) = <Helper as SubscriptionsBenchmarkHelper<_, _, _>>::price();
		let period = 100_000;
		let conditions = SubscriptionConditionsOf::<Runtime, Instance> {
			price: ItemPriceOf::<Runtime, Instance> { asset, amount },
			period,
			term: Some(u32::MAX),
			min_commitment: None,
			grace: period / 10,
		};
		assert_ok!(Listings::set_subscription_conditions(
			origin.clone(),
			inventory_id,
			0,
			conditions,
			None
		));
		assert_ok!(Listings::set_subscription_policy(
			origin,
			inventory_id,
			0,
			SubscriptionPolicy {
				amendments: AmendmentPolicy::WithNotice { periods: 1 },
				terminable_by_merchant: true,
			}
		));

		// The subscriber: the benchmarks skip when this origin cannot be produced.
		let origin =
			<<Runtime as Config<Instance>>::SubscribeOrigin as EnsureOriginWithArg<_, _>>::try_successful_origin(
				&inventory_id,
			)
			.expect("the subscriber origin is expected to be producible; qed");
		let who = <<Runtime as Config<Instance>>::SubscribeOrigin as EnsureOriginWithArg<_, _>>::try_origin(
			origin.clone(),
			&inventory_id,
		)
		.ok()
		.expect("the subscriber origin is expected to succeed; qed");
		<Helper as SubscriptionsBenchmarkHelper<_, _, _>>::fund(&who, &asset, amount);
		<<Runtime as Config<Instance>>::SubscriptionDeposit as frame_support::traits::Consideration<_, _>>::ensure_successful(
			&who,
			Listings::subscription_footprint(),
		);

		assert_ok!(Listings::subscribe(origin, inventory_id, 0));
	});
}
