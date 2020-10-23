#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_error, decl_event, decl_module, dispatch};
use frame_system::ensure_signed;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use valiu_node_commons::{Asset, Collateral};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type CollateralBalance<T> =
    <<T as Trait>::Collateral as MultiCurrency<<T as frame_system::Trait>::AccountId>>::Balance;
type MintMembers = pallet_membership::Instance0;
type ProviderMembers = pallet_membership::DefaultInstance;

pub trait Trait:
    pallet_membership::Trait<MintMembers> + pallet_membership::Trait<ProviderMembers>
{
    type Asset: MultiCurrency<
        Self::AccountId,
        Balance = <<Self as Trait>::Collateral as MultiCurrency<
            <Self as frame_system::Trait>::AccountId,
        >>::Balance,
        CurrencyId = Asset,
    >;
    type Collateral: MultiReservableCurrency<Self::AccountId, CurrencyId = Asset>;
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        Attestation(AccountId, Collateral),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
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
            balance: CollateralBalance<T>
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
    }
}
