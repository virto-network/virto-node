#![allow(
    clippy::unused_unit,
    unused_qualifications,
    missing_debug_implementations
)]
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
pub struct Rates<CurrencyId, BaseCurrencyId> {
    pub from: CurrencyId,
    pub to: BaseCurrencyId,
    pub method: PaymentMethod,
}

pub mod primitives {
    // NOTE We should be able to make our module generic over any kind of `PerThing`
    pub type LpRatePremium = sp_runtime::Percent;
}

#[frame_support::pallet]
pub mod pallet {
    use crate::{primitives::LpRatePremium, PaymentMethod, RateProvider, Rates};
    use frame_support::{
        dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Contains,
    };
    use frame_system::pallet_prelude::*;
    use orml_traits::DataProvider;
    use sp_runtime::traits::AtLeast32BitUnsigned;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Type of assets that the LP can set rates for
        type CurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord;
        /// Type of the base currency
        type BaseCurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord;
        /// type of Oracle Provider
        type OracleProvider: DataProvider<Self::CurrencyId, Self::OracleValue>;
        /// type of Oracle Value
        type OracleValue: Parameter + Member + Ord + AtLeast32BitUnsigned;
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
    // this only represents the basis points above/below the rates supplied by oracle
    // module
    pub(super) type RateStore<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        Rates<T::CurrencyId, T::BaseCurrencyId>,
        Twox64Concat,
        T::AccountId,
        LpRatePremium,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::metadata(
        T::AccountId = "AccountId",
        T::CurrencyId = "CurrencyId",
        T::BaseCurrencyId = "BaseCurrencyId"
    )]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Rate has been updated
        RatesUpdated(T::AccountId, T::CurrencyId, T::BaseCurrencyId),
        /// Rates have been removed by LP
        RatesRemoved(T::AccountId, T::CurrencyId, T::BaseCurrencyId),
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
            to: T::BaseCurrencyId,
            method: PaymentMethod,
            rate: LpRatePremium,
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
        /// Can be called when LPs want to opt-out from serving a pair
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn remove_price(
            origin: OriginFor<T>,
            from: T::CurrencyId,
            to: T::BaseCurrencyId,
            method: PaymentMethod,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            RateStore::<T>::remove(Rates { from, to, method }, &who);
            Self::deposit_event(Event::RatesRemoved(who, from, to));
            Ok(().into())
        }
    }

    impl<T: Config>
        RateProvider<T::CurrencyId, T::BaseCurrencyId, PaymentMethod, T::AccountId, T::OracleValue>
        for Pallet<T>
    {
        fn get_rates(
            from: T::CurrencyId,
            to: T::BaseCurrencyId,
            method: PaymentMethod,
            who: T::AccountId,
        ) -> Option<T::OracleValue> {
            let lp_premium = RateStore::<T>::get(Rates { from, to, method }, who)?;
            // asssuming that to(base-currency) is USD or any value thats common everywhere
            let oracle_rate = T::OracleProvider::get(&from)?;
            Some(lp_premium.mul_floor(oracle_rate.clone()) + oracle_rate)
        }
    }
}
