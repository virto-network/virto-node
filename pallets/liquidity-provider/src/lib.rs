#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_error, decl_event, decl_module, dispatch};
use frame_system::ensure_signed;
use orml_traits::MultiCurrency;
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type CurrencyIdOf<T> =
    <<T as Trait>::Currency as MultiCurrency<<T as frame_system::Trait>::AccountId>>::CurrencyId;
type BalanceOf<T> =
    <<T as Trait>::Currency as MultiCurrency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait Trait: pallet_membership::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Currency: MultiCurrency<Self::AccountId>;
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        CurrencyId = CurrencyIdOf<T>,
    {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        Attestation(AccountId, CurrencyId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        NotAProvider
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        #[weight = 0]
        pub fn attest(origin, asset_id: CurrencyIdOf<T>, balance: BalanceOf<T>)   -> dispatch::DispatchResult{
            let provider = ensure_signed(origin)?;

            let members = pallet_membership::Module::<T>::members();
            members.binary_search(&provider).map_err(|_| Error::<T>::NotAProvider)?;

            T::Currency::deposit(asset_id, &provider, balance)?;
            Self::deposit_event(RawEvent::Attestation(provider, asset_id));
            Ok(())
        }
    }
}
