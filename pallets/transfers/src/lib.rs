#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

use codec::FullCodec;
use frame_support::traits::Currency;
use orml_traits::MultiCurrency;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod primitives {}

pub trait CurrencyPair<T: Config>: FullCodec {
    type Origin: Currency<T::AccountId>;
    type Destination: Currency<T::AccountId>;
}
impl<T: Config, P, Q> CurrencyPair<T> for (P, Q)
where
    P: Currency<T::AccountId> + FullCodec,
    Q: Currency<T::AccountId> + FullCodec,
{
    type Origin = P;
    type Destination = Q;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::StaticLookup;

    type BalanceOf<T> =
        <<T as Config>::Assets as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
    type CurrencyIdOf<T> = <<T as Config>::Assets as MultiCurrency<
        <T as frame_system::Config>::AccountId,
    >>::CurrencyId;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Assets: MultiCurrency<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    //#[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId")]
    pub enum Event<T: Config> {}

    #[pallet::error]
    pub enum Error<T> {
        /// Transfers are not enabled yet.
        NotImplemented,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// A flexible transfer mechanism that allows sending assets to an accout with a different
        /// currency that might even be outside of the chain.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn transfer(
            origin: OriginFor<T>,
            _from_currency: CurrencyIdOf<T>,
            _to_currency: CurrencyIdOf<T>,
            _to: <T::Lookup as StaticLookup>::Source,
            _to_value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;
            Err(Error::<T>::NotImplemented.into())
        }
    }
}
