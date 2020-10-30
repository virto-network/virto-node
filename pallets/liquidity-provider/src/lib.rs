#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch};
use frame_system::ensure_signed;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use sp_arithmetic::traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};
use valiu_node_commons::{Asset, DistributionStrategy, OfferRate};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod transfer_handlers;

type Balance<T> =
    <<T as Trait>::Collateral as MultiCurrency<<T as frame_system::Trait>::AccountId>>::Balance;
type ProviderMembers = pallet_membership::DefaultInstance;

pub trait Trait: pallet_membership::Trait<ProviderMembers>
where
    Balance<Self>: CheckedAdd + CheckedDiv + CheckedMul + CheckedSub + From<u8>,
{
    type Asset: MultiCurrency<Self::AccountId, Balance = Balance<Self>, CurrencyId = Asset>;
    type Collateral: MultiReservableCurrency<Self::AccountId, CurrencyId = Asset>;
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        Balance = Balance<T>,
    {
        Attestation(AccountId, Asset),
        Transfer(AccountId, AccountId, Balance),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        MustNotBeUsdv,
        NoFunds
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call
    where
        origin: T::Origin
    {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 0]
        pub fn attest(
            origin,
            asset: Asset,
            balance: Balance<T>,
            new_offer_rates: Vec<OfferRate<Balance<T>>>
        ) -> dispatch::DispatchResult
        {
            match asset {
                Asset::Usdv => return Err(crate::Error::<T>::MustNotBeUsdv.into()),
                Asset::Collateral(collateral) => {
                    let who = ensure_signed(origin)?;
                    do_update_offer_rates::<T>(&who, asset, new_offer_rates);
                    do_attest::<T>(who.clone(), Asset::Usdv, balance)?;
                    T::Collateral::deposit(collateral.into(), &who, balance)?;
                    T::Collateral::reserve(collateral.into(), &who, balance)?;
                    Self::deposit_event(RawEvent::Attestation(who, collateral.into()));
                    Ok(())
                }
            }
        }

        #[weight = 0]
        pub fn transfer(
            origin,
            to: <T as frame_system::Trait>::AccountId,
            to_amount: Balance<T>,
            ds: DistributionStrategy
        ) -> dispatch::DispatchResult
        {
            let from = ensure_signed(origin)?;
            match ds {
                DistributionStrategy::Evenly => transfer_handlers::transfer_evenly::<T>(from, to, to_amount)?
            }
            Ok(())
        }

        #[weight = 0]
        pub fn update_offer_rates(
            origin,
            asset: Asset,
            new_offer_rates: Vec<OfferRate<Balance<T>>>
        ) -> dispatch::DispatchResult
        {
            let who = ensure_signed(origin)?;
            do_update_offer_rates::<T>(&who, asset, new_offer_rates);
            Ok(())
        }
    }
}

impl<T> Module<T>
where
    T: Trait,
{
    pub fn offer_rates(
        from: &<T as frame_system::Trait>::AccountId,
        asset: Asset,
    ) -> Vec<OfferRate<Balance<T>>> {
        <Offers<T>>::get(from, asset)
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Tokens {
        pub Offers get(fn accounts):
            double_map hasher(blake2_128_concat) <T as frame_system::Trait>::AccountId,
            hasher(twox_64_concat) Asset => Vec<OfferRate<Balance<T>>>
    }
}

#[inline]
fn do_attest<T>(
    from: <T as frame_system::Trait>::AccountId,
    asset: Asset,
    balance: Balance<T>,
) -> dispatch::DispatchResult
where
    T: Trait,
{
    pallet_membership::Module::<T, ProviderMembers>::members()
        .binary_search(&from)
        .ok()
        .ok_or(pallet_membership::Error::<T, ProviderMembers>::NotMember)?;
    T::Asset::deposit(asset, &from, balance)?;
    Module::<T>::deposit_event(RawEvent::Attestation(from, asset));
    Ok(())
}

#[inline]
fn do_update_offer_rates<T>(
    from: &<T as frame_system::Trait>::AccountId,
    asset: Asset,
    new_offer_rates: Vec<OfferRate<Balance<T>>>,
) where
    T: Trait,
{
    <Offers<T>>::mutate(from, asset, |old_offers| {
        for new_offer_rate in new_offer_rates {
            let idx = old_offers
                .binary_search_by(|e| e.asset().cmp(&new_offer_rate.asset()))
                .unwrap_or_else(|x| x);
            if let Some(rslt) = old_offers.get_mut(idx) {
                *rslt = new_offer_rate;
            } else {
                old_offers.insert(idx, new_offer_rate)
            }
        }
    })
}
