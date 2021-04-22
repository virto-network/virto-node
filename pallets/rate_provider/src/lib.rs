#![cfg_attr(not(feature = "std"), no_std)]
use codec::{Decode, Encode};
pub use pallet::*;
pub use vln_primitives::{PaymentMethod, RateProvider};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rates<CurrencyId> {
    pub from: CurrencyId,
    pub to: CurrencyId,
    pub method: PaymentMethod,
}

#[frame_support::pallet]
pub mod pallet {
    use crate::{PaymentMethod, RateProvider, Rates};
    use frame_support::{
        dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Contains,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::FixedU128;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Type of assets that the LP can set rates for
        type CurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord;
        /// Whitelist of LPs allowed to participate, this will eventually be removed and
        /// anyone should be able to publish rates
        type Whitelist: Contains<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn rates)]
    // Rates provided by an LP, the actual rate is not stored here
    // this only represents the 
    pub(super) type RateStore<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        Rates<T::CurrencyId>,
        Twox64Concat,
        T::AccountId,
        FixedU128,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId", T::CurrencyId = "CurrencyId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Rate has been updated
        RatesUpdated(T::AccountId, T::CurrencyId, T::CurrencyId),
        /// Rates have been removed by LP
        RatesRemoved(T::AccountId, T::CurrencyId, T::CurrencyId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Caller is not permitted to create/update rates.
        NotPermitted,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Update the price for a pair<>paymentmethod in storage
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn update_price(
            origin: OriginFor<T>,
            from: T::CurrencyId,
            to: T::CurrencyId,
            method: PaymentMethod,
            rate: FixedU128,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // restrict calls to whitelisted LPs only
            ensure!(T::Whitelist::contains(&who), Error::<T>::NotPermitted);
            // Update storage.
            RateStore::<T>::insert(Rates { from, to, method }, &who, rate);
            Self::deposit_event(Event::RatesUpdated(who, from, to));
            Ok(().into())
        }

        /// Remove any exising price stored by LP
        /// Can be called when LPs want to optout from serving a pair
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn remove_price(
            origin: OriginFor<T>,
            from: T::CurrencyId,
            to: T::CurrencyId,
            method: PaymentMethod,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            RateStore::<T>::remove(Rates { from, to, method }, &who);
            Self::deposit_event(Event::RatesRemoved(who, from, to));
            Ok(().into())
        }
    }

    impl<T: Config> RateProvider<T::CurrencyId, PaymentMethod, T::AccountId, FixedU128> for Pallet<T> {
        fn get_rates(
            from: T::CurrencyId,
            to: T::CurrencyId,
            method: PaymentMethod,
            who: T::AccountId,
        ) -> Option<FixedU128> {
            RateStore::<T>::get(Rates { from, to, method }, who)
        }
    }
}
