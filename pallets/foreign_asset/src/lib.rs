#![allow(
    clippy::unused_unit,
    unused_qualifications,
    missing_debug_implementations
)]
#![cfg_attr(not(feature = "std"), no_std)]
use orml_traits::{MultiCurrency, MultiLockableCurrency};
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use orml_traits::LockIdentifier;

    type BalanceOf<T> =
        <<T as Config>::Assets as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
    type CurrencyIdOf<T> = <<T as Config>::Assets as MultiCurrency<
        <T as frame_system::Config>::AccountId,
    >>::CurrencyId;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Assets: MultiLockableCurrency<Self::AccountId>;
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Whitelist: EnsureOrigin<Self::Origin, Success = Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId")]
    pub enum Event<T: Config> {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        Attestation(T::AccountId, CurrencyIdOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {}

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    const LOCK_ID: LockIdentifier = *b"_foreign";

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Attest the existance of an asset off-chain in a permissioned way
        /// The dispatch origin of this call must be `Whitelist`
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn attest(
            origin: OriginFor<T>,
            currency: CurrencyIdOf<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = T::Whitelist::ensure_origin(origin)?;
            T::Assets::deposit(currency, &who, amount)?;
            // Assuming attested assets can't be transfered since moving them
            // to a different owner doesn't mean they moved in the real world
            T::Assets::set_lock(LOCK_ID, currency, &who, amount)?;
            Self::deposit_event(Event::Attestation(who, currency));
            Ok(().into())
        }
    }
}
