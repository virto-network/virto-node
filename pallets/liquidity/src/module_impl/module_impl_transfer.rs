use crate::{Balance, Module, RawEvent, Trait};
use arrayvec::ArrayVec;
use core::cmp::Ordering;
use frame_support::dispatch;
use orml_traits::{currency::BalanceStatus, MultiCurrency, MultiReservableCurrency};
use sp_arithmetic::traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Zero};
use vln_commons::{Asset, Collateral};

type Balances<T> = ArrayVec<[Account<T>; Collateral::len()]>;

impl<T> Module<T>
where
    T: Trait,
{
    #[inline]
    pub(crate) fn transfer_evenly(
        from: <T as frame_system::Trait>::AccountId,
        to: <T as frame_system::Trait>::AccountId,
        to_amount: Balance<T>,
    ) -> dispatch::DispatchResult {
        let orig = Self::collateral_balances(&from);
        let mut modified = Self::collateral_balances(&from);
        let actually_transferred =
            if let Some(rslt) = Self::test_transfer_evenly(&mut modified, to_amount) {
                rslt
            } else {
                return Err(crate::Error::<T>::NoFunds.into());
            };
        Self::do_transfer_evenly(actually_transferred, from, modified, orig, to)?;
        Ok(())
    }

    // Checks `wants_to_transfer` and returns the sum of all collaterals
    #[inline]
    fn check_if_usdv_to_transfer_is_greater_than_total_collaterals(
        collateral_balances: &Balances<T>,
        wants_to_transfer: Balance<T>,
    ) -> Option<Balance<T>> {
        let mut total_collaterals: Balance<T> = Balance::<T>::zero();

        for Account { reserved, .. } in collateral_balances.iter() {
            total_collaterals = total_collaterals.checked_add(reserved)?;
        }

        if wants_to_transfer > total_collaterals {
            return None;
        }

        Some(total_collaterals)
    }

    // Creates a snapshot with all non-empty collateral balances of a given user
    #[inline]
    fn collateral_balances(who: &<T as frame_system::Trait>::AccountId) -> Balances<T> {
        Collateral::variants()
            .iter()
            .filter_map(|&collateral| {
                let reserved = T::Collateral::reserved_balance(collateral.into(), who);

                if reserved == Balance::<T>::zero() {
                    None
                } else {
                    Some(Account {
                        asset: collateral.into(),
                        reserved,
                    })
                }
            })
            .collect()
    }

    // Takes the original and subtracted balances and issue transfer operations
    #[inline]
    fn do_transfer_evenly(
        amount: Balance<T>,
        from: <T as frame_system::Trait>::AccountId,
        modified: Balances<T>,
        original: Balances<T>,
        to: <T as frame_system::Trait>::AccountId,
    ) -> dispatch::DispatchResult {
        for (a, b) in modified.iter().zip(original.iter()) {
            let diff = b
                .reserved
                .checked_sub(&a.reserved)
                .ok_or(crate::Error::<T>::NoFunds)?;
            T::Collateral::repatriate_reserved(a.asset, &from, &to, diff, BalanceStatus::Reserved)?;
        }
        T::Asset::transfer(Asset::Usdv, &from, &to, amount)?;
        Module::<T>::deposit_event(RawEvent::Transfer(from, to, amount));
        Ok(())
    }

    // Just check if a given set of collateral balances can fulfil the desired amount of USDv by
    // subtracting each asset
    #[inline]
    fn test_transfer_evenly(
        collateral_balances: &mut Balances<T>,
        wants_to_transfer: Balance<T>,
    ) -> Option<Balance<T>> {
        let total_collaterals = Self::check_if_usdv_to_transfer_is_greater_than_total_collaterals(
            &collateral_balances,
            wants_to_transfer,
        )?;

        let mut actually_transfered: Balance<T> = Balance::<T>::zero();
        let mut last_actually_transfered: Balance<T> = Balance::<T>::zero();
        let hundred = Balance::<T>::from(100u32);
        let pct = wants_to_transfer
            .checked_mul(&hundred)?
            .checked_div(&total_collaterals)?;

        loop {
            for Account { reserved, .. } in collateral_balances.iter_mut() {
                let partial_collateral = reserved.checked_mul(&pct)?.checked_div(&hundred)?;
                let partial_actually_transfered =
                    actually_transfered.checked_add(&partial_collateral)?;

                match partial_actually_transfered.cmp(&wants_to_transfer) {
                    Ordering::Equal => {
                        actually_transfered =
                            actually_transfered.checked_add(&partial_collateral)?;
                        *reserved = reserved.checked_sub(&partial_collateral)?;
                        return Some(actually_transfered);
                    }
                    Ordering::Greater => {
                        let diff = wants_to_transfer.checked_sub(&actually_transfered)?;
                        actually_transfered = actually_transfered.checked_add(&diff)?;
                        *reserved = reserved.checked_sub(&diff)?;
                        return Some(actually_transfered);
                    }
                    Ordering::Less => {
                        actually_transfered =
                            actually_transfered.checked_add(&partial_collateral)?;
                        *reserved = reserved.checked_sub(&partial_collateral)?;
                    }
                }
            }

            let partial_collateral_can_not_fulfill_wants_to_transfer =
                actually_transfered == last_actually_transfered;
            if partial_collateral_can_not_fulfill_wants_to_transfer {
                return Some(actually_transfered);
            } else {
                last_actually_transfered = actually_transfered;
            }
        }
    }
}

struct Account<T>
where
    T: Trait
{
    asset: <<T as Trait>::Collateral as MultiCurrency<<T as frame_system::Trait>::AccountId>>::CurrencyId,
    reserved: Balance<T>
}
