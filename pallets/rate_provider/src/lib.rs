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

// SBP1 review
/// Missing weight (as you know), I would suggest to implement them early on


#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Contains,
    };
    use frame_system::pallet_prelude::*;
    use orml_traits::DataProvider;
    use sp_runtime::FixedPointNumber;
    use sp_std::collections::btree_map::BTreeMap;
    use vln_primitives::{
        AssetPair, PaymentMethod, RateCombinator, RateDetail, RatePremiumType, RateProvider,
    };

    pub type RateDetailOf<T> = RateDetail<<T as Config>::OracleValue, RatePremiumType>;

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
        type OracleValue: Parameter + Member + FixedPointNumber;
        /// Whitelist of LPs allowed to participate, this will eventually be removed and
        /// anyone should be able to publish rates
        type Whitelist: Contains<Self::AccountId>;
        /// Implementation of RatePremiumCalc that handles combining premium to
        /// rate
        type RateCombinator: RateCombinator<Self::OracleValue, RatePremiumType>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn rates)]
    // Rates published by providers, the actual rate is not stored here
    // this only represents the basis points above/below the rates supplied by oracle
    // module
    pub(super) type Rates<T: Config> = StorageDoubleMap<
        _,
// SBP1 Externally feeded, might consider crypto hashes
        Twox64Concat,
        AssetPair<T::Asset, T::BaseAsset>,
        Twox64Concat,
        PaymentMethod,
        BTreeMap<T::AccountId, RateDetailOf<T>>,
        ValueQuery,
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
            rate: RateDetailOf<T>,
        ) -> DispatchResultWithPostInfo {
// SBP1 No need to use DispatchResultWithPostInfo if always returning default
            let who = ensure_signed(origin)?;
            // restrict calls to whitelisted LPs only
            ensure!(T::Whitelist::contains(&who), Error::<T>::NotPermitted);
            // Update storage.
            Rates::<T>::try_mutate(
                AssetPair { base, quote },
                medium,
                |providers| -> DispatchResult {
                    providers.insert(who.clone(), rate);
                    Ok(())
                },
            )?;
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
            Rates::<T>::try_mutate(
                AssetPair { base, quote },
                medium,
                |providers| -> DispatchResult {
                    providers.remove(&who);
                    Ok(())
                },
            )?;
// SBP1 What if removing when no rate is provided? Shall the event be updated?
            Self::deposit_event(Event::RatesRemoved(who, base, quote));
            Ok(().into())
        }
    }

    impl<T: Config> RateProvider<AssetPair<T::Asset, T::BaseAsset>, PaymentMethod, T::AccountId>
        for Pallet<T>
    {
        type Rate = T::OracleValue;

        fn get_rates(
            pair: AssetPair<T::Asset, T::BaseAsset>,
            medium: PaymentMethod,
            who: T::AccountId,
        ) -> Option<T::OracleValue> {
            let rate_map = Rates::<T>::get(pair.clone(), medium);
            let rate = rate_map.get(&who)?;
            // asssuming that quote (base-currency) is USD or any value thats common between the price-feed
            // and the provider
            match rate {
                // return the fixed price added by the provider
                RateDetail::Fixed(fx) => Some(*fx),
                // combine the premium of the provider to oracle price
                RateDetail::Premium(px) => {
                    let oracle_rate = T::PriceFeed::get(&pair.base)?;
                    Some(T::RateCombinator::combine_rates(oracle_rate, *px))
                }
            }
        }
    }
}
