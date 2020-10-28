use crate::{Balance, Module, RawEvent, Trait};
use arrayvec::ArrayVec;
use core::cmp::Ordering;
use frame_support::dispatch;
use orml_traits::{currency::BalanceStatus, MultiCurrency, MultiReservableCurrency};
use sp_arithmetic::traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Zero};
use valiu_node_commons::{Asset, Collateral};

type Balances<T> = ArrayVec<[Account<T>; Collateral::len()]>;

#[inline]
pub fn transfer_evenly<T>(
    from: <T as frame_system::Trait>::AccountId,
    to: <T as frame_system::Trait>::AccountId,
    to_amount: Balance<T>,
) -> dispatch::DispatchResult
where
    T: Trait,
{
    let orig = collateral_balances::<T>(&from);
    let mut modified = collateral_balances::<T>(&from);
    let actually_transferred = if let Some(rslt) = test_transfer_evenly(&mut modified, to_amount) {
        rslt
    } else {
        return Err(crate::Error::<T>::NoFunds.into());
    };
    do_transfer_evenly(actually_transferred, from, modified, orig, to)?;
    Ok(())
}

struct Account<T>
where
    T: Trait
{
    asset: <<T as Trait>::Collateral as MultiCurrency<<T as frame_system::Trait>::AccountId>>::CurrencyId,
    reserved: Balance<T>
}

// Checks `wants_to_transfer` and returns the sum of all collaterals
#[inline]
fn check_if_usdv_to_transfer_is_greater_than_total_collaterals<T>(
    collateral_balances: &Balances<T>,
    wants_to_transfer: Balance<T>,
) -> Option<Balance<T>>
where
    T: Trait,
{
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
fn collateral_balances<T>(who: &<T as frame_system::Trait>::AccountId) -> Balances<T>
where
    T: Trait,
{
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
fn do_transfer_evenly<T>(
    amount: Balance<T>,
    from: <T as frame_system::Trait>::AccountId,
    modified: Balances<T>,
    original: Balances<T>,
    to: <T as frame_system::Trait>::AccountId,
) -> dispatch::DispatchResult
where
    T: Trait,
{
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
fn test_transfer_evenly<T>(
    collateral_balances: &mut Balances<T>,
    wants_to_transfer: Balance<T>,
) -> Option<Balance<T>>
where
    T: Trait,
{
    let total_collaterals = check_if_usdv_to_transfer_is_greater_than_total_collaterals(
        &collateral_balances,
        wants_to_transfer,
    )?;

    let mut actually_transfered: Balance<T> = Balance::<T>::zero();
    let mut last_actually_transfered: Balance<T> = Balance::<T>::zero();
    let _100 = Balance::<T>::from(100);
    let pct = wants_to_transfer
        .checked_mul(&_100)?
        .checked_div(&total_collaterals)?;

    loop {
        for Account { reserved, .. } in collateral_balances.iter_mut() {
            let partial_collateral = reserved.checked_mul(&pct)?.checked_div(&_100)?;
            let partial_actually_transfered =
                actually_transfered.checked_add(&partial_collateral)?;

            match partial_actually_transfered.cmp(&wants_to_transfer) {
                Ordering::Equal => {
                    actually_transfered = actually_transfered.checked_add(&partial_collateral)?;
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
                    actually_transfered = actually_transfered.checked_add(&partial_collateral)?;
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
