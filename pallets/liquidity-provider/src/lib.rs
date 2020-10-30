#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch};
use frame_system::ensure_signed;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use sp_arithmetic::traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};
use valiu_node_commons::{AccountRate, Asset, DistributionStrategy, OfferRate};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod transfer_handlers;

type AccountRateTy<T> = AccountRate<<T as frame_system::Trait>::AccountId, Balance<T>>;
type Balance<T> =
    <<T as Trait>::Collateral as MultiCurrency<<T as frame_system::Trait>::AccountId>>::Balance;
type OfferRateTy<T> = OfferRate<Balance<T>>;
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
            offer_rates: Vec<OfferRateTy<T>>
        ) -> dispatch::DispatchResult
        {
            match asset {
                Asset::Usdv => return Err(crate::Error::<T>::MustNotBeUsdv.into()),
                Asset::Collateral(collateral) => {
                    let who = ensure_signed(origin)?;
                    update_account_rates::<T>(&who, asset, offer_rates);
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
            offer_rates: Vec<OfferRateTy<T>>
        ) -> dispatch::DispatchResult
        {
            let who = ensure_signed(origin)?;
            update_account_rates::<T>(&who, asset, offer_rates);
            Ok(())
        }
    }
}

impl<T> Module<T>
where
    T: Trait,
{
    pub fn account_rates(from_asset: &Asset, to_asset: &Asset) -> Vec<AccountRateTy<T>> {
        <Offers<T>>::get(from_asset, to_asset)
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Tokens {
        pub Offers get(fn accounts):
            double_map hasher(twox_64_concat) Asset,
            hasher(twox_64_concat) Asset => Vec<AccountRateTy<T>>
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
fn update_account_rates<T>(
    from: &<T as frame_system::Trait>::AccountId,
    asset: Asset,
    offer_rates: Vec<OfferRateTy<T>>,
) where
    T: Trait,
{
    for offer_rate in offer_rates {
        <Offers<T>>::mutate(asset, offer_rate.asset(), |account_rates| {
            let idx = account_rates
                .binary_search_by(|el| el.account().cmp(from))
                .unwrap_or_else(|idx| idx);
            if let Some(rslt) = account_rates.get_mut(idx) {
                *rslt.rate_mut() = *offer_rate.rate();
            } else {
                account_rates.insert(idx, AccountRate::new(from.clone(), *offer_rate.rate()))
            }
        })
    }
}
