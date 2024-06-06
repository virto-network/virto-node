// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Auxiliary struct/enums for parachain runtimes.
//! Taken from polkadot/runtime/common (at a21cd64) and adapted for parachains.

use frame_support::traits::{
	fungibles::{Balanced, Credit},
	Currency, Imbalance, OnUnbalanced,
};
use pallet_asset_tx_payment::HandleCredit;
use sp_runtime::traits::MaybeEquivalence;
use sp_std::marker::PhantomData;
use xcm::latest::Location;

// TODO - Create and import XCM common types
//use xcm::latest::{AssetId, Fungibility::Fungible, MultiAsset, MultiLocation};

/// Type alias to conveniently refer to the `Currency::NegativeImbalance`
/// associated type.
pub type NegativeImbalance<T> =
	<pallet_balances::Pallet<T> as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

/// Type alias to conveniently refer to `frame_system`'s `Config::AccountId`.
pub type AccountIdOf<R> = <R as frame_system::Config>::AccountId;

pub struct DealWithFees<R>(PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for DealWithFees<R>
where
	R: pallet_balances::Config + pallet_collator_selection::Config + pallet_treasury::Config,
	pallet_treasury::Pallet<R>: OnUnbalanced<NegativeImbalance<R>>,
	AccountIdOf<R>: From<polkadot_core_primitives::v2::AccountId> + Into<polkadot_core_primitives::v2::AccountId>,
	<R as frame_system::Config>::RuntimeEvent: From<pallet_balances::Event<R>>,
{
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance<R>>) {
		use pallet_treasury::Pallet as Treasury;
		if let Some(mut fees) = fees_then_tips.next() {
			if let Some(tips) = fees_then_tips.next() {
				tips.merge_into(&mut fees);
			}
			// 100% of the fees + tips (if any) go to the treasury
			<Treasury<R> as OnUnbalanced<_>>::on_unbalanced(fees);
		}
	}
}

/// A `HandleCredit` implementation that naively transfers the fees to the block
/// author. Will drop and burn the assets in case the transfer fails.
pub struct AssetsToBlockAuthor<R, I>(PhantomData<(R, I)>);
impl<R, I> HandleCredit<AccountIdOf<R>, pallet_assets::Pallet<R, I>> for AssetsToBlockAuthor<R, I>
where
	I: 'static,
	R: pallet_authorship::Config + pallet_assets::Config<I>,
	AccountIdOf<R>: From<polkadot_core_primitives::v2::AccountId> + Into<polkadot_core_primitives::v2::AccountId>,
{
	fn handle_credit(credit: Credit<AccountIdOf<R>, pallet_assets::Pallet<R, I>>) {
		if let Some(author) = pallet_authorship::Pallet::<R>::author() {
			// In case of error: Will drop the result triggering the `OnDrop` of the
			// imbalance.
			let _ = pallet_assets::Pallet::<R, I>::resolve(&author, credit);
		}
	}
}

pub struct AsAssetMultiLocation<AssetId, AssetIdInfoGetter>(PhantomData<(AssetId, AssetIdInfoGetter)>);
impl<AssetId, AssetIdInfoGetter> MaybeEquivalence<Location, AssetId>
	for AsAssetMultiLocation<AssetId, AssetIdInfoGetter>
where
	AssetId: Clone,
	AssetIdInfoGetter: AssetMultiLocationGetter<AssetId>,
{
	fn convert(asset_multi_location: &Location) -> Option<AssetId> {
		AssetIdInfoGetter::get_asset_id(asset_multi_location)
	}

	fn convert_back(asset_id: &AssetId) -> Option<Location> {
		AssetIdInfoGetter::get_asset_multi_location(asset_id.clone())
	}
}

pub trait AssetMultiLocationGetter<AssetId> {
	fn get_asset_multi_location(asset_id: AssetId) -> Option<Location>;
	fn get_asset_id(asset_multi_location: &Location) -> Option<AssetId>;
}
