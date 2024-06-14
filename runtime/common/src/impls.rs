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
	fungible::{DecreaseIssuance, IncreaseIssuance},
	fungibles::{Balanced, Credit},
	Currency, Imbalance, OnUnbalanced,
};
use pallet_asset_tx_payment::HandleCredit;
// use pallet_balances::Pallet as Balances;
use pallet_treasury::Pallet as Treasury;
use sp_std::marker::PhantomData;

// TODO - Create and import XCM common types
//use xcm::latest::{AssetId, Fungibility::Fungible, MultiAsset, MultiLocation};

/// Type alias to conveniently refer to `frame_system`'s `Config::AccountId`.
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

// /// Type alias to conveniently refer to the `Currency::NegativeImbalance`
// /// associated type.
pub type NegativeImbalance<T, I> = <pallet_balances::Pallet<T, I> as Currency<AccountIdOf<T>>>::NegativeImbalance;

/// Type Alias to represent fungible imbalances
pub type FungibleImbalance<T, I> = frame_support::traits::fungible::Imbalance<
	<T as pallet_balances::Config<I>>::Balance,
	DecreaseIssuance<AccountIdOf<T>, pallet_balances::Pallet<T, I>>,
	IncreaseIssuance<AccountIdOf<T>, pallet_balances::Pallet<T, I>>,
>;

/// [OnUnbalanced] handler that takes 100% of the fees + tips (if any), and
/// makes them go to the treasury.
pub struct DealWithFees<T, I: 'static = ()>(PhantomData<(T, I)>);

impl<T, I: 'static, IB> OnUnbalanced<IB> for DealWithFees<T, I>
where
	T: pallet_balances::Config<I> + pallet_treasury::Config<I> + pallet_collator_selection::Config,
	Treasury<T, I>: OnUnbalanced<IB>,
	IB: Imbalance<<T as pallet_balances::Config<I>>::Balance>,
{
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = IB>) {
		if let Some(mut fees) = fees_then_tips.next() {
			if let Some(tips) = fees_then_tips.next() {
				tips.merge_into(&mut fees);
				Treasury::<T, I>::on_unbalanced(fees);
			}
		}
	}
}

/// [OnUnbalanced] handler that takes 100% of the fees + tips (if any), and
/// makes them go to the treasury.
pub struct DealWithFungibleFees<T, I: 'static = ()>(PhantomData<(T, I)>);

impl<T, I: 'static> OnUnbalanced<FungibleImbalance<T, I>> for DealWithFungibleFees<T, I>
where
	T: pallet_balances::Config<I> + pallet_treasury::Config<I> + pallet_collator_selection::Config,
	Treasury<T, I>: OnUnbalanced<FungibleImbalance<T, I>>,
{
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = FungibleImbalance<T, I>>) {
		if let Some(mut fees) = fees_then_tips.next() {
			if let Some(tips) = fees_then_tips.next() {
				tips.merge_into(&mut fees);
				Treasury::<T, I>::on_unbalanced(fees);
			}
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
