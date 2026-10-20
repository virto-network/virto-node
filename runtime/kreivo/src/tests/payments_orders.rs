//! The payments and orders benchmarks create `Here(0)`, the default asset id, and mint small
//! amounts of it. In benchmark builds `Here(0)` is an `Assets` asset under the native-and-assets
//! union, so those setups fund accounts without hitting the native existential deposit. These tests
//! run the `pay` setups of both pallets in the runtime, because their benchmarks cannot run from a
//! test externalities.

use super::*;
use crate::{
	config::{payments::NativeAndAssets, RelaychainData},
	Assets, Orders, Payments, System,
};
use frame_support::traits::{
	fungible::Mutate as _,
	fungibles::{Create, Mutate},
};
use sp_runtime::traits::BlockNumberProvider;

fn relay_block(n: u32) {
	System::set_block_number(n);
	RelaychainData::set_block_number(n);
}

/// The setup of the payments benchmarks: a default asset, funded for both parties.
#[test]
fn payments_benchmarks_setup_runs_in_the_runtime() {
	TestExternalities::default().execute_with(|| {
		relay_block(1);
		let (sender, beneficiary) = (AccountId32::new([10; 32]), AccountId32::new([11; 32]));
		let asset = FungibleAssetLocation::default();
		assert_ok!(<NativeAndAssets as Create<_>>::create(asset, sender.clone(), true, 1));
		for who in [&sender, &beneficiary] {
			assert_ok!(<NativeAndAssets as Mutate<_>>::mint_into(asset, who, 10_000_000));
		}
		assert_ok!(Payments::pay(
			RuntimeOrigin::signed(sender.clone()),
			CommunityLookup::unlookup(beneficiary.clone()),
			asset,
			100_000,
			None
		));
		let payment_id = pallet_payments::Payment::<Runtime>::iter_key_prefix(&sender)
			.next()
			.expect("the payment is created");
		assert_ok!(Payments::release(RuntimeOrigin::signed(sender), payment_id));
	})
}

/// The setup of the orders `pay` benchmark: a priced item in the default asset, and a buyer funded
/// in it, who checks out and pays.
#[test]
fn orders_benchmarks_setup_runs_in_the_runtime() {
	use frame_contrib_traits::listings::{
		item::{ItemPrice, Mutate as ItemMutate},
		InventoryLifecycle,
	};
	use frame_support::traits::EnsureOrigin;
	use pallet_orders::BenchmarkHelper;

	type Helper = <Runtime as pallet_orders::Config<pallet_listings::Instance1>>::BenchmarkHelper;
	type CreateOrigin = <Runtime as pallet_orders::Config<pallet_listings::Instance1>>::CreateOrigin;

	TestExternalities::default().execute_with(|| {
		relay_block(1);
		let asset = FungibleAssetLocation::default();
		assert_ok!(<Assets as Create<_>>::create(
			asset,
			AccountId32::new([1; 32]),
			false,
			1
		));

		let owner = AccountId32::new([2; 32]);
		assert_ok!(Balances::mint_into(&owner, 2 * UNITS));
		let inventory_id = <Helper as BenchmarkHelper<_, _>>::inventory_id();
		assert_ok!(<crate::Listings as InventoryLifecycle<_>>::create(inventory_id, &owner));
		let item_id = <Helper as BenchmarkHelper<_, _>>::item_id(0);
		assert_ok!(<crate::Listings as ItemMutate<_>>::publish(
			&inventory_id,
			&item_id,
			b"".to_vec(),
			Some(ItemPrice { asset, amount: 10 })
		));

		let buyer = AccountId32::new([3; 32]);
		let origin = RuntimeOrigin::signed(buyer.clone());
		let _ = CreateOrigin::try_origin(origin.clone());
		assert_ok!(Balances::mint_into(&buyer, UNITS));
		assert_ok!(Assets::mint_into(asset, &buyer, 1_000_000));
		let order_id = pallet_orders::NextOrderId::<Runtime, pallet_listings::Instance1>::get();
		assert_ok!(Orders::create_cart(origin.clone(), None));
		assert_ok!(Orders::set_cart_items(
			origin.clone(),
			order_id,
			vec![((inventory_id, item_id), Some(AccountId32::new([4; 32])))]
		));
		assert_ok!(Orders::checkout(origin.clone(), order_id));
		assert_ok!(Orders::pay(origin, order_id));
	})
}
