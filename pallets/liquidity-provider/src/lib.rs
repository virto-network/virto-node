#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_error, decl_event, decl_module, dispatch};
use frame_system::ensure_signed;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use sp_arithmetic::traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};
use valiu_node_commons::{Asset, Collateral, DistributionStrategy};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod transfer_handlers;

type Balance<T> =
    <<T as Trait>::Collateral as MultiCurrency<<T as frame_system::Trait>::AccountId>>::Balance;
type MintMembers = pallet_membership::Instance0;
type ProviderMembers = pallet_membership::DefaultInstance;

pub trait Trait:
    pallet_membership::Trait<MintMembers> + pallet_membership::Trait<ProviderMembers>
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
        Attestation(AccountId, Collateral),
        Transfer(AccountId, AccountId, Balance),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
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
            collateral: Collateral,
            balance: Balance<T>
        ) -> dispatch::DispatchResult
        {
            let who = ensure_signed(origin)?;

            pallet_membership::Module::<T, MintMembers>::members()
                .binary_search(&who)
                .ok()
                .ok_or(pallet_membership::Error::<T, MintMembers>::NotMember)?;

            pallet_membership::Module::<T, ProviderMembers>::members()
                .binary_search(&who)
                .ok()
                .ok_or(pallet_membership::Error::<T, ProviderMembers>::NotMember)?;

            T::Collateral::deposit(Asset::Collateral(collateral), &who, balance)?;
            T::Collateral::reserve(Asset::Collateral(collateral), &who, balance)?;
            T::Asset::deposit(Asset::Usdv, &who, balance)?;
            Self::deposit_event(RawEvent::Attestation(who, collateral));
            Ok(())
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
    }
}
