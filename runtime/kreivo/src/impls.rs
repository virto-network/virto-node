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

use super::*;
use cumulus_primitives_core::{relay_chain::BlockNumber as RelayBlockNumber, DmpMessageHandler};
use frame_support::{
	traits::{Contains, Currency},
	weights::Weight,
};

use pallet_lockdown_mode::impls::PauseXcmExecution;
use sp_runtime::DispatchResult;

/// Type alias to conveniently refer to the `Currency::NegativeImbalance`
/// associated type.
pub type NegativeImbalance<T> =
	<pallet_balances::Pallet<T> as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

/// Type alias to conveniently refer to `frame_system`'s `Config::AccountId`.
pub type AccountIdOf<R> = <R as frame_system::Config>::AccountId;

pub struct RuntimeBlackListedCalls;
impl Contains<RuntimeCall> for RuntimeBlackListedCalls {
	fn contains(call: &RuntimeCall) -> bool {
		match call {
			RuntimeCall::Balances(_) => false,
			RuntimeCall::Treasury(_) => false,
			RuntimeCall::Utility(_) => false,
			RuntimeCall::Assets(_) => false,
			RuntimeCall::Multisig(_) => false,
			_ => true,
		}
	}
}

pub struct LockdownDmpHandler;
impl DmpMessageHandler for LockdownDmpHandler {
	fn handle_dmp_messages(_iter: impl Iterator<Item = (RelayBlockNumber, Vec<u8>)>, limit: Weight) -> Weight {
		DmpQueue::handle_dmp_messages(_iter, limit)
	}
}

pub struct XcmExecutionManager {}
impl PauseXcmExecution for XcmExecutionManager {
	fn suspend_xcm_execution() -> DispatchResult {
		XcmpQueue::suspend_xcm_execution(RuntimeOrigin::root())
	}
	fn resume_xcm_execution() -> DispatchResult {
		XcmpQueue::resume_xcm_execution(RuntimeOrigin::root())
	}
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug, MaxEncodedLen, scale_info::TypeInfo,
)]
pub enum ProxyType {
	/// Fully permissioned proxy. Can execute any call on behalf of _proxied_.
	Any,
	/// Can execute any call that does not transfer funds or assets.
	NonTransfer,
	/// Proxy with the ability to reject time-delay proxy announcements.
	CancelProxy,
	/// Assets proxy. Can execute any call from `assets`, **including asset
	/// transfers**.
	Assets,
	/// Owner proxy. Can execute calls related to asset ownership.
	AssetOwner,
	/// Asset manager. Can execute calls related to asset management.
	AssetManager,
	/// Collator selection proxy. Can execute calls related to collator
	/// selection mechanism.
	Collator,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}

impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => !matches!(
				c,
				RuntimeCall::Balances { .. }
					| RuntimeCall::Assets { .. }
					| RuntimeCall::Nfts { .. }
					| RuntimeCall::Uniques { .. }
			),
			ProxyType::CancelProxy => matches!(
				c,
				RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. })
					| RuntimeCall::Utility { .. }
					| RuntimeCall::Multisig { .. }
			),
			ProxyType::Assets => {
				matches!(
					c,
					RuntimeCall::Assets { .. }
						| RuntimeCall::Utility { .. }
						| RuntimeCall::Multisig { .. }
						| RuntimeCall::Nfts { .. }
						| RuntimeCall::Uniques { .. }
				)
			}
			ProxyType::AssetOwner => matches!(
				c,
				RuntimeCall::Assets(TrustBackedAssetsCall::create { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::start_destroy { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::destroy_accounts { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::destroy_approvals { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::finish_destroy { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::transfer_ownership { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::set_team { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::set_metadata { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::clear_metadata { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::create { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::destroy { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::redeposit { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::transfer_ownership { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::set_team { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::set_collection_max_supply { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::lock_collection { .. })
					| RuntimeCall::Uniques(pallet_uniques::Call::create { .. })
					| RuntimeCall::Uniques(pallet_uniques::Call::destroy { .. })
					| RuntimeCall::Uniques(pallet_uniques::Call::transfer_ownership { .. })
					| RuntimeCall::Uniques(pallet_uniques::Call::set_team { .. })
					| RuntimeCall::Uniques(pallet_uniques::Call::set_metadata { .. })
					| RuntimeCall::Uniques(pallet_uniques::Call::set_attribute { .. })
					| RuntimeCall::Uniques(pallet_uniques::Call::set_collection_metadata { .. })
					| RuntimeCall::Uniques(pallet_uniques::Call::clear_metadata { .. })
					| RuntimeCall::Uniques(pallet_uniques::Call::clear_attribute { .. })
					| RuntimeCall::Uniques(pallet_uniques::Call::clear_collection_metadata { .. })
					| RuntimeCall::Uniques(pallet_uniques::Call::set_collection_max_supply { .. })
					| RuntimeCall::Utility { .. }
					| RuntimeCall::Multisig { .. }
			),
			ProxyType::AssetManager => matches!(
				c,
				RuntimeCall::Assets(TrustBackedAssetsCall::mint { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::burn { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::freeze { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::thaw { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::freeze_asset { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::thaw_asset { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::force_mint { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::update_mint_settings { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::mint_pre_signed { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::set_attributes_pre_signed { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::lock_item_transfer { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::unlock_item_transfer { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::lock_item_properties { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::set_metadata { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::clear_metadata { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::set_collection_metadata { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::clear_collection_metadata { .. })
					| RuntimeCall::Uniques(pallet_uniques::Call::mint { .. })
					| RuntimeCall::Uniques(pallet_uniques::Call::burn { .. })
					| RuntimeCall::Uniques(pallet_uniques::Call::freeze { .. })
					| RuntimeCall::Uniques(pallet_uniques::Call::thaw { .. })
					| RuntimeCall::Uniques(pallet_uniques::Call::freeze_collection { .. })
					| RuntimeCall::Uniques(pallet_uniques::Call::thaw_collection { .. })
					| RuntimeCall::Utility { .. }
					| RuntimeCall::Multisig { .. }
			),
			ProxyType::Collator => matches!(
				c,
				RuntimeCall::CollatorSelection { .. } | RuntimeCall::Utility { .. } | RuntimeCall::Multisig { .. }
			),
		}
	}

	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::Assets, ProxyType::AssetOwner) => true,
			(ProxyType::Assets, ProxyType::AssetManager) => true,
			(ProxyType::NonTransfer, ProxyType::Collator) => true,
			_ => false,
		}
	}
}
