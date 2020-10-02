#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch};
use frame_system::ensure_root;
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Trait: frame_system::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {
        Providers: Vec<T::AccountId>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        ProviderRegistered(AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        AlreadyRegistered,
        NoneValue,
        StorageOverflow,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 0]
        pub fn register(origin, account: T::AccountId) -> dispatch::DispatchResult {
            ensure_root(origin)?;

            let mut providers = Providers::<T>::get();

            match providers.binary_search(&account) {
                Ok(_) => Err(Error::<T>::AlreadyRegistered.into()),
                Err(idx) => {
                    providers.insert(idx, account.clone());
                    Providers::<T>::put(providers);
                    Self::deposit_event(RawEvent::ProviderRegistered(account));
                    Ok(())
                }
            }
        }
    }
}
