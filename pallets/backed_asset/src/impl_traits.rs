use crate::{Config, Pallet};
use frame_support::traits::Currency;
use sp_runtime::DispatchError;

// Allow backed-asset to be used as a Currency wrapping the underlying base currency
impl<T: Config<I>, I: 'static> Currency<T::AccountId> for Pallet<T, I> {
    type Balance = T::Balance;
    type PositiveImbalance = <T::BaseCurrency as Currency<T::AccountId>>::PositiveImbalance;
    type NegativeImbalance = <T::BaseCurrency as Currency<T::AccountId>>::NegativeImbalance;

    fn total_balance(who: &T::AccountId) -> Self::Balance {
        T::BaseCurrency::total_balance(who)
    }
    fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool {
        T::BaseCurrency::can_slash(who, value)
    }
    fn total_issuance() -> Self::Balance {
        T::BaseCurrency::total_issuance()
    }
    fn minimum_balance() -> Self::Balance {
        T::BaseCurrency::minimum_balance()
    }
    fn burn(amount: Self::Balance) -> Self::PositiveImbalance {
        T::BaseCurrency::burn(amount)
    }
    fn issue(amount: Self::Balance) -> Self::NegativeImbalance {
        T::BaseCurrency::issue(amount)
    }
    fn free_balance(who: &T::AccountId) -> Self::Balance {
        T::BaseCurrency::free_balance(who)
    }
    fn ensure_can_withdraw(
        who: &T::AccountId,
        amount: Self::Balance,
        reasons: frame_support::traits::WithdrawReasons,
        new_balance: Self::Balance,
    ) -> sp_runtime::DispatchResult {
        T::BaseCurrency::ensure_can_withdraw(who, amount, reasons, new_balance)
    }

    fn transfer(
        source: &T::AccountId,
        dest: &T::AccountId,
        value: Self::Balance,
        existence_requirement: frame_support::traits::ExistenceRequirement,
    ) -> sp_runtime::DispatchResult {
        T::BaseCurrency::transfer(source, dest, value, existence_requirement)
    }

    fn slash(who: &T::AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
        T::BaseCurrency::slash(who, value)
    }
    fn deposit_into_existing(
        who: &T::AccountId,
        value: Self::Balance,
    ) -> Result<Self::PositiveImbalance, DispatchError> {
        T::BaseCurrency::deposit_into_existing(who, value)
    }
    fn deposit_creating(who: &T::AccountId, value: Self::Balance) -> Self::PositiveImbalance {
        T::BaseCurrency::deposit_creating(who, value)
    }
    fn withdraw(
        who: &T::AccountId,
        value: Self::Balance,
        reasons: frame_support::traits::WithdrawReasons,
        liveness: frame_support::traits::ExistenceRequirement,
    ) -> Result<Self::NegativeImbalance, DispatchError> {
        T::BaseCurrency::withdraw(who, value, reasons, liveness)
    }
    fn make_free_balance_be(
        who: &T::AccountId,
        balance: Self::Balance,
    ) -> frame_support::traits::SignedImbalance<Self::Balance, Self::PositiveImbalance> {
        T::BaseCurrency::make_free_balance_be(who, balance)
    }
}
