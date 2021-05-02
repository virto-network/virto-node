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
    use orml_traits::{MultiCurrency, MultiReservableCurrency};
    use sp_runtime::{FixedPointNumber, FixedU128};
    use vln_primitives::*;

    type BalanceOf<T> =
        <<T as Config>::Asset as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
    type CurrencyIdOf<T> =
        <<T as Config>::Asset as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;
    type RateOf<T> = PairPrice<AssetPair<CurrencyIdOf<T>, CurrencyIdOf<T>>, FixedU128>;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Type of assets that can be swapped
        type Asset: MultiReservableCurrency<Self::AccountId>;
        /// Rate provider trait
        type RateProvider: RateProvider<
            AssetPair<CurrencyIdOf<Self>, CurrencyIdOf<Self>>,
            PaymentMethod,
            Self::AccountId,
        >;
        /// Whitelist of admins allowed to arbitrate in case of conflicts
        type Whitelist: Contains<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn rates)]
    // Swaps created mapped by useraccount and nonce
    pub(super) type Swaps<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        T::AccountId,
        Twox64Concat,
        u32,
        Swap<T::AccountId, RateOf<T>, BalanceOf<T>>,
    >;

    /// Current swap index for a user
    #[pallet::storage]
    #[pallet::getter(fn swap_index)]
    pub(super) type SwapIndex<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    //#[pallet::metadata(T::AccountId = "AccountId")]
    pub enum Event<T: Config> {
        /// A new swap has been created
        SwapCreated(T::AccountId, SwapKind),
        /// Existing swap has been updated
        SwapUpdated(T::AccountId, SwapKind),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        InvalidProvider,
        /// Swap does not exist
        InvalidSwap,
        /// Action not permitted by user
        ActionNotPermitted,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Allow any user to open a swap-in request
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_swap_in(
            origin: OriginFor<T>,
            base: CurrencyIdOf<T>,
            quote: CurrencyIdOf<T>,
            method: PaymentMethod,
            amount: BalanceOf<T>,
            human: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let swap_nonce = SwapIndex::<T>::get(who.clone()) + 1;
            let pair = AssetPair { base, quote };
            let _price = T::RateProvider::get_rates(pair.clone(), method, human.clone())
                .ok_or_else(|| Error::<T>::InvalidProvider)?;
            Swaps::<T>::insert(
                who.clone(),
                swap_nonce,
                Swap {
                    human,
                    kind: SwapKind::In(SwapIn::Created),
                    price: PairPrice {
                        pair,
                        price: FixedU128::zero(), // TODO: insert actual price
                    },
                    amount,
                },
            );
            SwapIndex::<T>::insert(who.clone(), swap_nonce);
            Self::deposit_event(Event::SwapCreated(who, SwapKind::In(SwapIn::Created)));
            Ok(().into())
        }

        /// this extrinsic allows the provider to accept/reject/complete swap requests
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn provider_process_swap_in(
            origin: OriginFor<T>,
            owner: T::AccountId,
            swap_id: u32,
            state: SwapIn,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Swaps::<T>::try_mutate(owner, swap_id, |maybe_swap| -> DispatchResult {
                let swap = maybe_swap.take().ok_or(Error::<T>::InvalidSwap)?;
                // ensure the caller is the assigned provider
                ensure!(swap.human == who, Error::<T>::ActionNotPermitted);
                // only allow provider to accept/reject/complete
                match state {
                    SwapIn::Rejected(_) => {
                        // ensure the swap is in created state
                        ensure!(
                            swap.kind == SwapKind::In(SwapIn::Created),
                            Error::<T>::ActionNotPermitted
                        );
                        *maybe_swap = Some(Swap {
                            kind: SwapKind::In(state.clone()),
                            ..swap
                        });
                    }
                    SwapIn::Accepted(_) => {
                        // ensure the swap is in created state
                        ensure!(
                            swap.kind == SwapKind::In(SwapIn::Created),
                            Error::<T>::ActionNotPermitted
                        );
                        // reserve the amount to cash_in, to be released on confirmation
                        T::Asset::reserve(swap.price.pair.quote, &who, swap.amount)?; // TODO: multiply with price
                        *maybe_swap = Some(Swap {
                            kind: SwapKind::In(state.clone()),
                            ..swap
                        });
                    }
                    _ => *maybe_swap = Some(swap),
                };
                Ok(())
            })?;
            Self::deposit_event(Event::SwapUpdated(who, SwapKind::In(state)));
            Ok(().into())
        }

        /// this extrinsic allows the provider to accept/reject swap requests
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn confirm_swap_in(
            origin: OriginFor<T>,
            swap_id: u32,
            proof: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Swaps::<T>::try_mutate(who.clone(), swap_id, |maybe_swap| -> DispatchResult {
                let swap = maybe_swap.take().ok_or(Error::<T>::InvalidSwap)?;
                // ensure the swap has been accepted by the provider
                match swap.kind {
                    SwapKind::In(SwapIn::Accepted(_)) => {
                        *maybe_swap = Some(Swap {
                            kind: SwapKind::In(SwapIn::Confirmed(proof.clone())),
                            ..swap
                        });
                    }
                    _ => *maybe_swap = Some(swap),
                };
                Ok(())
            })?;
            Self::deposit_event(Event::SwapUpdated(
                who,
                SwapKind::In(SwapIn::Confirmed(proof)),
            ));
            Ok(().into())
        }

        /// this extrinsic allows the provider to complete swapin requests and release
        /// the amount to user wallet
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn complete_swap_in(
            origin: OriginFor<T>,
            owner: T::AccountId,
            swap_id: u32,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Swaps::<T>::try_mutate(owner.clone(), swap_id, |maybe_swap| -> DispatchResult {
                let swap = maybe_swap.take().ok_or(Error::<T>::InvalidSwap)?;
                // ensure the caller is the assigned provider
                ensure!(swap.human == who, Error::<T>::ActionNotPermitted);
                // ensure the swap has been confirmed by the owner
                match swap.kind {
                    SwapKind::In(SwapIn::Confirmed(_)) => {
                        T::Asset::unreserve(swap.price.pair.quote, &who, swap.amount);
                        T::Asset::transfer(swap.price.pair.quote, &who, &owner, swap.amount)?;
                        *maybe_swap = Some(Swap {
                            kind: SwapKind::In(SwapIn::Completed),
                            ..swap
                        });
                    }
                    _ => *maybe_swap = Some(swap),
                };
                Ok(())
            })?;
            Self::deposit_event(Event::SwapUpdated(who, SwapKind::In(SwapIn::Completed)));
            Ok(().into())
        }

        /// Allow any user to open a swap-out request
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_swap_out(
            origin: OriginFor<T>,
            base: CurrencyIdOf<T>,
            quote: CurrencyIdOf<T>,
            method: PaymentMethod,
            amount: BalanceOf<T>,
            human: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let swap_nonce = SwapIndex::<T>::get(who.clone()) + 1;
            let pair = AssetPair { base, quote };
            let _price = T::RateProvider::get_rates(pair.clone(), method, human.clone())
                .ok_or_else(|| Error::<T>::InvalidProvider)?;
            // reserve the user balance to swap out
            T::Asset::reserve(quote, &who, amount)?; // TODO: mul with actual price
            Swaps::<T>::insert(
                who.clone(),
                swap_nonce,
                Swap {
                    human,
                    kind: SwapKind::Out(SwapOut::Created),
                    price: PairPrice {
                        pair,
                        price: FixedU128::zero(), // TODO: insert actual price
                    },
                    amount,
                },
            );
            SwapIndex::<T>::insert(who.clone(), swap_nonce);
            Self::deposit_event(Event::SwapCreated(who, SwapKind::Out(SwapOut::Created)));
            Ok(().into())
        }

        /// this extrinsic allows the provider to accept/confirm swap requests
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn provider_process_swap_out(
            origin: OriginFor<T>,
            owner: T::AccountId,
            swap_id: u32,
            state: SwapOut,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Swaps::<T>::try_mutate(owner, swap_id, |maybe_swap| -> DispatchResult {
                let mut swap = maybe_swap.take().ok_or(Error::<T>::InvalidSwap)?;
                // ensure the caller is the assigned provider
                ensure!(swap.human == who, Error::<T>::ActionNotPermitted);
                // only allow provider to reject/confirm
                match state {
                    SwapOut::Rejected(_) => {
                        // ensure the swap is in created state
                        ensure!(
                            swap.kind == SwapKind::Out(SwapOut::Created),
                            Error::<T>::ActionNotPermitted
                        );
                        swap.kind = SwapKind::Out(state.clone());
                        Ok(())
                    }
                    SwapOut::Confirmed(_) => {
                        // ensure the swap is in created state
                        ensure!(
                            swap.kind == SwapKind::Out(SwapOut::Created),
                            Error::<T>::ActionNotPermitted
                        );
                        swap.kind = SwapKind::Out(state.clone());
                        Ok(())
                    }
                    _ => Err(Error::<T>::ActionNotPermitted.into()),
                }
            })?;
            Self::deposit_event(Event::SwapUpdated(who, SwapKind::Out(state)));
            Ok(().into())
        }

        /// this extrinsic allows the user to complete swapout requests and release
        /// the amount to provider wallet
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn complete_swap_out(origin: OriginFor<T>, swap_id: u32) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Swaps::<T>::try_mutate(who.clone(), swap_id, |maybe_swap| -> DispatchResult {
                let mut swap = maybe_swap.take().ok_or(Error::<T>::InvalidSwap)?;
                // ensure the swap has been confirmed by the provider
                match swap.kind {
                    SwapKind::Out(SwapOut::Confirmed(_)) => {
                        T::Asset::unreserve(swap.price.pair.quote, &who, swap.amount);
                        T::Asset::transfer(swap.price.pair.quote, &who, &swap.human, swap.amount)?;
                        swap.kind = SwapKind::Out(SwapOut::Completed);
                        Ok(())
                    }
                    _ => Err(Error::<T>::ActionNotPermitted.into()),
                }
            })?;
            Self::deposit_event(Event::SwapUpdated(who, SwapKind::Out(SwapOut::Completed)));
            Ok(().into())
        }
    }
}
