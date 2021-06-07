#![allow(clippy::all, unused)]
use super::*;
use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::DispatchError,
    traits::{
        fungibles::{Inspect, InspectHold, MutateHold, Transfer},
        tokens::{DepositConsequence, WithdrawConsequence},
    },
};

pub struct MockAssets;
impl MutateHold<AccountId> for MockAssets {
    fn hold(asset: Self::AssetId, who: &AccountId, amount: Self::Balance) -> DispatchResult {
        Ok(())
    }

    fn release(
        asset: Self::AssetId,
        who: &AccountId,
        amount: Self::Balance,
        best_effort: bool,
    ) -> Result<Self::Balance, DispatchError> {
        Ok(amount)
    }

    fn transfer_held(
        asset: Self::AssetId,
        source: &AccountId,
        dest: &AccountId,
        amount: Self::Balance,
        best_effort: bool,
        on_hold: bool,
    ) -> Result<Self::Balance, DispatchError> {
        Ok(amount)
    }
}

impl Inspect<AccountId> for MockAssets {
    /// Means of identifying one asset class from another.
    type AssetId = u32;

    /// Scalar type for representing balance of an account.
    type Balance = u64;

    /// The total amount of issuance in the system.
    fn total_issuance(asset: Self::AssetId) -> Self::Balance {
        0u64
    }

    /// The minimum balance any single account may have.
    fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
        0u64
    }

    /// Get the `asset` balance of `who`.
    fn balance(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
        0u64
    }

    /// Get the maximum amount of `asset` that `who` can withdraw/transfer successfully.
    fn reducible_balance(asset: Self::AssetId, who: &AccountId, keep_alive: bool) -> Self::Balance {
        0u64
    }

    /// Returns `true` if the `asset` balance of `who` may be increased by `amount`.
    fn can_deposit(
        asset: Self::AssetId,
        who: &AccountId,
        amount: Self::Balance,
    ) -> DepositConsequence {
        DepositConsequence::Success
    }

    /// Returns `Failed` if the `asset` balance of `who` may not be decreased by `amount`, otherwise
    /// the consequence.
    fn can_withdraw(
        asset: Self::AssetId,
        who: &AccountId,
        amount: Self::Balance,
    ) -> WithdrawConsequence<Self::Balance> {
        WithdrawConsequence::Success
    }
}

impl InspectHold<AccountId> for MockAssets {
    /// Amount of funds held in hold.
    fn balance_on_hold(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
        0u64
    }

    /// Check to see if some `amount` of `asset` may be held on the account of `who`.
    fn can_hold(asset: Self::AssetId, who: &AccountId, amount: Self::Balance) -> bool {
        false
    }
}

impl Transfer<AccountId> for MockAssets {
    /// Transfer funds from one account into another.
    fn transfer(
        asset: Self::AssetId,
        source: &AccountId,
        dest: &AccountId,
        amount: Self::Balance,
        keep_alive: bool,
    ) -> Result<Self::Balance, DispatchError> {
        Ok(amount)
    }
}
