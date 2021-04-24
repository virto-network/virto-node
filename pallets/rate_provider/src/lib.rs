#![allow(
    clippy::unused_unit,
    unused_qualifications,
    missing_debug_implementations
)]
#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Contains,
    };
    use frame_system::pallet_prelude::*;
    use orml_traits::DataProvider;
    use vln_primitives::{AssetPair, PaymentMethod, RatePremiumType, RateProvider, Rates};

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Type of assets that the LP can set rates for
        type Asset: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord;
        /// Type of the base currency
        type BaseAsset: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord;
        /// type of Oracle Provider
        type PriceFeed: DataProvider<Self::Asset, Self::OracleValue>;
        /// type of Oracle Value
        type OracleValue: Parameter + Member + Ord;
        /// Whitelist of LPs allowed to participate, this will eventually be removed and
        /// anyone should be able to publish rates
        type Whitelist: Contains<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn rates)]
    // Rates published by providers, the actual rate is not stored here
    // this only represents the basis points above/below the rates supplied by oracle
    // module
    pub(super) type RateStore<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        Rates<T::Asset, T::BaseAsset>,
        Twox64Concat,
        T::AccountId,
        RatePremiumType,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::metadata(
        T::AccountId = "AccountId",
        T::Asset = "Asset",
        T::BaseAsset = "BaseAsset"
    )]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Rate has been updated
        RatesUpdated(T::AccountId, T::Asset, T::BaseAsset),
        /// Rates have been removed by LP
        RatesRemoved(T::AccountId, T::Asset, T::BaseAsset),
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
            base: T::Asset,
            quote: T::BaseAsset,
            medium: PaymentMethod,
            rate: RatePremiumType,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // restrict calls to whitelisted LPs only
            ensure!(T::Whitelist::contains(&who), Error::<T>::NotPermitted);
            // Update storage.
            RateStore::<T>::insert(
                Rates {
                    pair: AssetPair { base, quote },
                    medium,
                },
                &who,
                rate,
            );
            Self::deposit_event(Event::RatesUpdated(who, base, quote));
            Ok(().into())
        }

        /// Remove any exising price stored by LP
        /// Can be called when LPs want to opt-out from serving a pair
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn remove_price(
            origin: OriginFor<T>,
            base: T::Asset,
            quote: T::BaseAsset,
            medium: PaymentMethod,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            RateStore::<T>::remove(
                Rates {
                    pair: AssetPair { base, quote },
                    medium,
                },
                &who,
            );
            Self::deposit_event(Event::RatesRemoved(who, base, quote));
            Ok(().into())
        }
    }

    impl<T: Config>
        RateProvider<
            AssetPair<T::Asset, T::BaseAsset>,
            PaymentMethod,
            T::AccountId,
            T::OracleValue,
            RatePremiumType,
        > for Pallet<T>
    {
        fn get_rates(
            pair: AssetPair<T::Asset, T::BaseAsset>,
            medium: PaymentMethod,
            who: T::AccountId,
        ) -> Option<(T::OracleValue, RatePremiumType)> {
            let lp_premium = RateStore::<T>::get(
                Rates {
                    pair: pair.clone(),
                    medium,
                },
                &who,
            )?;
            // asssuming that quote (base-currency) is USD or any value thats common between the price-feed
            // and the provider
            let oracle_rate = T::PriceFeed::get(&pair.base)?;
            Some((oracle_rate, lp_premium))
        }
    }
}
