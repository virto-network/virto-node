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
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use orml_traits::{MultiCurrency, MultiReservableCurrency};
    use vln_primitives::{EscrowDetail, EscrowHandler, EscrowId, EscrowState};

    type BalanceOf<T> =
        <<T as Config>::Asset as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
    type AssetIdOf<T> =
        <<T as Config>::Asset as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// the type of assets this pallet can hold in escrow
        type Asset: MultiReservableCurrency<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn rates)]
    /// Escrows created by a user, this method of storageDoubleMap is chosen since there is no usecase for
    /// listing escrows by provider/currency. The escrow will only be referenced by the creator in
    /// any transaction of interest.
    pub(super) type Escrow<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        T::AccountId,
        Twox64Concat,
        EscrowId,
        EscrowDetail<T::AccountId, AssetIdOf<T>, BalanceOf<T>>,
    >;

    /// Current escrow index for a user
    #[pallet::storage]
    #[pallet::getter(fn swap_index)]
    pub(super) type EscrowIndex<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, EscrowId, ValueQuery>;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Rate has been updated
        EscrowCreated(T::AccountId, AssetIdOf<T>, BalanceOf<T>),
        /// Rates have been removed by LP
        EscrowReleased(T::AccountId, EscrowId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The selected escrow does not exist
        InvalidEscrow,
        /// The selected escrow cannot be released
        EscrowAlreadyReleased,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// This allows any user to create a new escrow, that releases only to specified recipent
        /// The only action is to store the details of this escrow in storage and reserve
        /// the specified amount.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_escrow(
            origin: OriginFor<T>,
            recipent: T::AccountId,
            asset: AssetIdOf<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            <Self as EscrowHandler<T::AccountId, AssetIdOf<T>, BalanceOf<T>>>::create_escrow(
                who, recipent, asset, amount,
            )?;
            Ok(().into())
        }

        /// Release any created escrow, this will transfer the reserved amount from the
        /// creator of the escrow to the assigned recipent
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn release_escrow(
            origin: OriginFor<T>,
            escrow_id: EscrowId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            <Self as EscrowHandler<T::AccountId, AssetIdOf<T>, BalanceOf<T>>>::release_escrow(
                who, escrow_id,
            )?;
            Ok(().into())
        }
    }

    impl<T: Config> EscrowHandler<T::AccountId, AssetIdOf<T>, BalanceOf<T>> for Pallet<T> {
        fn create_escrow(
            from: T::AccountId,
            recipent: T::AccountId,
            asset: AssetIdOf<T>,
            amount: BalanceOf<T>,
        ) -> Result<EscrowId, DispatchError> {
            // try to reserve the amount in the user balance
            T::Asset::reserve(asset, &from, amount)?;
            let escrow_index = EscrowIndex::<T>::get(from.clone()) + 1;
            // add the escrow detail to storage
            Escrow::<T>::insert(
                from.clone(),
                escrow_index,
                EscrowDetail {
                    recipent,
                    asset,
                    amount,
                    state: EscrowState::Created,
                },
            );
            // update the escrow index
            EscrowIndex::<T>::insert(from.clone(), escrow_index);
            Self::deposit_event(Event::EscrowCreated(from, asset, amount));
            Ok(escrow_index)
        }

        fn release_escrow(from: T::AccountId, escrow_id: EscrowId) -> Result<(), DispatchError> {
            // add the escrow detail to storage
            Escrow::<T>::try_mutate(from.clone(), escrow_id, |maybe_escrow| -> DispatchResult {
                let escrow = maybe_escrow.take().ok_or(Error::<T>::InvalidEscrow)?;
                // ensure the escrow is in created state
                ensure!(
                    escrow.state == EscrowState::Created,
                    Error::<T>::EscrowAlreadyReleased
                );
                // unreserve the amount from the owner account
                T::Asset::unreserve(escrow.asset, &from, escrow.amount);
                // try to transfer the amount to recipent
                T::Asset::transfer(escrow.asset, &from, &escrow.recipent, escrow.amount)?;
                *maybe_escrow = Some(EscrowDetail {
                    state: EscrowState::Released,
                    ..escrow
                });
                Ok(())
            })?;
            Self::deposit_event(Event::EscrowReleased(from, escrow_id));
            Ok(())
        }

        fn get_escrow_details(
            from: T::AccountId,
            escrow_id: EscrowId,
        ) -> Option<EscrowDetail<T::AccountId, AssetIdOf<T>, BalanceOf<T>>> {
            Escrow::<T>::get(from, escrow_id)
        }
    }
}
