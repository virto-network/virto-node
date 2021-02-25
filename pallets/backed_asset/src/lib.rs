#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

mod impl_traits;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod primitives {
    pub type Share = sp_arithmetic::Permill;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use primitives::*;

    use codec::FullCodec;
    use frame_support::{
        dispatch::{DispatchResult, DispatchResultWithPostInfo},
        pallet_prelude::*,
        traits::Currency,
    };
    use frame_system::pallet_prelude::*;
    use orml_traits::{MultiCurrency, MultiReservableCurrency};
    use sp_arithmetic::traits::{AtLeast32BitUnsigned, Zero};
    use sp_std::fmt::Debug;

    type CurrencyIdOf<T, I> = <<T as Config<I>>::Collateral as MultiCurrency<
        <T as frame_system::Config>::AccountId,
    >>::CurrencyId;

    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        type Collateral: MultiReservableCurrency<Self::AccountId, Balance = Self::Balance>;
        type BaseCurrency: Currency<Self::AccountId, Balance = Self::Balance>;
        type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;
        type Balance: AtLeast32BitUnsigned
            + FullCodec
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + Default;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T, I = ()>(_);

    #[pallet::storage]
    #[pallet::getter(fn account_share)]
    pub type AccountShare<T: Config<I>, I: 'static = ()> =
        StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, CurrencyIdOf<T, I>, Share>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId")]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        Mint(T::AccountId, CurrencyIdOf<T, I>),
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Account doesn't have the needed amount of collateral.
        NotEnoughCollateral,
        /// The transaction was submitted with an invalid amount.
        InvalidAmount,
    }

    #[pallet::hooks]
    impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {}

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// Use some valid collateral to create the same amount of backed-assets updating the
        /// share ratio of the collateral compared to other collaterals backing the same asset.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn mint(
            origin: OriginFor<T>,
            collateral: CurrencyIdOf<T, I>,
            amount: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(!amount.is_zero(), Error::<T, I>::InvalidAmount);
            ensure!(
                T::Collateral::can_reserve(collateral, &who, amount),
                Error::<T, I>::NotEnoughCollateral
            );

            Self::update_account_shares(&who, collateral, amount)?;
            Self::deposit_into_existing(&who, amount)?;
            T::Collateral::reserve(collateral, &who, amount)?;

            Self::deposit_event(Event::Mint(who, collateral));
            Ok(().into())
        }
    }

    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        fn update_account_shares(
            who: &T::AccountId,
            collateral: CurrencyIdOf<T, I>,
            amount: T::Balance,
        ) -> DispatchResult {
            if !AccountShare::<T, I>::contains_key(who, collateral) {
                AccountShare::<T, I>::insert(who, collateral, Share::zero());
            }
            let new_balance = Self::free_balance(who) + amount;
            for (c, mut price) in Self::collateral_prices(who) {
                AccountShare::<T, I>::try_mutate(who, c, |share| -> DispatchResult {
                    if c == collateral {
                        price += amount;
                    };
                    *share = Some(Share::from_rational_approximation(price, new_balance));
                    Ok(())
                })?;
            }
            Ok(())
        }

        fn collateral_prices(
            who: &T::AccountId,
        ) -> impl Iterator<Item = (CurrencyIdOf<T, I>, T::Balance)> {
            let total = Self::free_balance(who);
            AccountShare::<T, I>::iter_prefix(who).map(move |(c, s)| (c, s * total))
        }
    }
}
