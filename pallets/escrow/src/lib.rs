#![allow(
    clippy::unused_unit,
    unused_qualifications,
    missing_debug_implementations
)]
#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

// Restore once mock is replaced by pallet-assets-freezer
// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        pallet_prelude::*,
        traits::{
            fungibles::{Inspect, MutateHold, Transfer},
            Contains,
        },
    };
    use frame_system::pallet_prelude::*;
    use vln_primitives::{EscrowDetail, EscrowHandler, EscrowState};

    type BalanceOf<T> =
        <<T as Config>::Asset as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
    type AssetIdOf<T> =
        <<T as Config>::Asset as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// the type of assets this pallet can hold in escrow
        type Asset: MutateHold<Self::AccountId>;
        /// whitelist of users allowed to settle disputes
        type JudgeWhitelist: Contains<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn rates)]
    /// Escrows created by a user, this method of storageDoubleMap is chosen since there is no usecase for
    /// listing escrows by provider/currency. The escrow will only be referenced by the creator in
    /// any transaction of interest.
    /// The storage map keys are the creator and the recipent, this also ensures
    /// that for any (sender,recipent) combo, only a single escrow is active. The history of escrow is not stored.
    pub(super) type Escrow<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId, // escrow creator
        Blake2_128Concat,
        T::AccountId, // escrow recipent
        EscrowDetail<AssetIdOf<T>, BalanceOf<T>>,
    >;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new escrow has been created
        EscrowCreated(T::AccountId, AssetIdOf<T>, BalanceOf<T>),
        /// Escrow amount released to the recipent
        EscrowReleased(T::AccountId, T::AccountId),
        /// Escrow has been cancelled by the creatot
        EscrowCancelled(T::AccountId, T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The selected escrow does not exist
        InvalidEscrow,
        /// The selected escrow cannot be released
        EscrowAlreadyReleased,
        /// The selected escrow already exists and is in process
        EscrowAlreadyInProcess,
        /// Action permitted only for whitelisted users
        InvalidAction,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// This allows any user to create a new escrow, that releases only to specified recipent
        /// The only action is to store the details of this escrow in storage and reserve
        /// the specified amount.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create(
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
        pub fn release(origin: OriginFor<T>, to: T::AccountId) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            <Self as EscrowHandler<T::AccountId, AssetIdOf<T>, BalanceOf<T>>>::release_escrow(
                who, to,
            )?;
            Ok(().into())
        }

        /// Cancel an escrow in created state, this will release the reserved back to
        /// creator of the escrow. This extrinsic can only be called by the recipent
        /// of the escrow
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn cancel(origin: OriginFor<T>, creator: T::AccountId) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            <Self as EscrowHandler<T::AccountId, AssetIdOf<T>, BalanceOf<T>>>::cancel_escrow(
                creator, who, // the caller must be the provider, creator cannot cancel
            )?;
            Ok(().into())
        }

        /// Allow admins to set state of an escrow
        /// This extrinsic is used to resolve disputes between the creator and
        /// recipent of the escrow.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn resolve(
            origin: OriginFor<T>,
            from: T::AccountId,
            recipent: T::AccountId,
            new_state: EscrowState,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // ensure the caller is part of the whitelist
            ensure!(T::JudgeWhitelist::contains(&who), Error::<T>::InvalidAction);
            // try to update the escrow to new state
            match new_state {
                EscrowState::Cancelled => {
                    <Self as EscrowHandler<T::AccountId, AssetIdOf<T>, BalanceOf<T>>>::cancel_escrow(
                        from, recipent,
                    )
                }
                EscrowState::Released => <Self as EscrowHandler<
                    T::AccountId,
                    AssetIdOf<T>,
                    BalanceOf<T>,
                >>::release_escrow(from, recipent),
                EscrowState::Created => Err(Error::<T>::InvalidAction.into()),
            }?;
            Ok(().into())
        }
    }

    impl<T: Config> EscrowHandler<T::AccountId, AssetIdOf<T>, BalanceOf<T>> for Pallet<T> {
        fn create_escrow(
            from: T::AccountId,
            recipent: T::AccountId,
            asset: AssetIdOf<T>,
            amount: BalanceOf<T>,
        ) -> Result<(), DispatchError> {
            Escrow::<T>::try_mutate(from.clone(), recipent, |maybe_escrow| -> DispatchResult {
                let new_escrow = Some(EscrowDetail {
                    asset,
                    amount,
                    state: EscrowState::Created,
                });
                match maybe_escrow {
                    Some(x) => {
                        // do not overwrite an in-process escrow!
                        // ensure the escrow is not in created state, it should
                        // be in released/cancelled, in which case it can be overwritten
                        ensure!(
                            x.state != EscrowState::Created,
                            Error::<T>::EscrowAlreadyInProcess
                        );
                        // reserve the amount from the escrow creator
                        T::Asset::hold(asset, &from, amount)?;
                        *maybe_escrow = new_escrow
                    }
                    None => {
                        // reserve the amount from the escrow creator
                        T::Asset::hold(asset, &from, amount)?;
                        *maybe_escrow = new_escrow
                    }
                }
                Ok(())
            })
        }

        fn release_escrow(from: T::AccountId, to: T::AccountId) -> Result<(), DispatchError> {
            // add the escrow detail to storage
            Escrow::<T>::try_mutate(from.clone(), to.clone(), |maybe_escrow| -> DispatchResult {
                let escrow = maybe_escrow.take().ok_or(Error::<T>::InvalidEscrow)?;
                // ensure the escrow is in created state
                ensure!(
                    escrow.state == EscrowState::Created,
                    Error::<T>::EscrowAlreadyReleased
                );
                // unreserve the amount from the owner account
                T::Asset::release(escrow.asset, &from, escrow.amount, false)?;
                // try to transfer the amount to recipent
// SP1 Can the release fail? Shall this be handled?
                T::Asset::transfer(escrow.asset, &from, &to, escrow.amount, true)?;
                *maybe_escrow = Some(EscrowDetail {
                    state: EscrowState::Released,
                    ..escrow
                });
                Ok(())
            })?;
            Self::deposit_event(Event::EscrowReleased(from, to));
            Ok(())
        }

        fn cancel_escrow(from: T::AccountId, to: T::AccountId) -> Result<(), DispatchError> {
            // add the escrow detail to storage
            Escrow::<T>::try_mutate(from.clone(), to.clone(), |maybe_escrow| -> DispatchResult {
                let escrow = maybe_escrow.take().ok_or(Error::<T>::InvalidEscrow)?;
                // ensure the escrow is in created state
                ensure!(
                    escrow.state == EscrowState::Created,
                    Error::<T>::EscrowAlreadyReleased
                );
                // unreserve the amount from the owner account
                T::Asset::release(escrow.asset, &from, escrow.amount, false)?;
                *maybe_escrow = Some(EscrowDetail {
                    state: EscrowState::Cancelled,
                    ..escrow
                });
                Ok(())
            })?;
            Self::deposit_event(Event::EscrowReleased(from, to));
            Ok(())
        }

        fn get_escrow_details(
            from: T::AccountId,
            to: T::AccountId,
        ) -> Option<EscrowDetail<AssetIdOf<T>, BalanceOf<T>>> {
            Escrow::<T>::get(from, to)
        }
    }
}
