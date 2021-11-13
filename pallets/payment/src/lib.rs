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
    use virto_primitives::{PaymentDetail, PaymentHandler, PaymentState};

    type BalanceOf<T> =
        <<T as Config>::Asset as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
    type AssetIdOf<T> =
        <<T as Config>::Asset as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// the type of assets this pallet can hold in payment
        type Asset: MultiReservableCurrency<Self::AccountId>;
        /// whitelist of users allowed to settle disputes
        type JudgeWhitelist: Contains<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn rates)]
    /// Payments created by a user, this method of storageDoubleMap is chosen since there is no usecase for
    /// listing payments by provider/currency. The payment will only be referenced by the creator in
    /// any transaction of interest.
    /// The storage map keys are the creator and the recipient, this also ensures
    /// that for any (sender,recipient) combo, only a single payment is active. The history of payment is not stored.
    pub(super) type Payment<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId, // payment creator
        Blake2_128Concat,
        T::AccountId, // payment recipient
        PaymentDetail<AssetIdOf<T>, BalanceOf<T>>,
    >;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new payment has been created
        PaymentCreated(T::AccountId, AssetIdOf<T>, BalanceOf<T>),
        /// Payment amount released to the recipient
        PaymentReleased(T::AccountId, T::AccountId),
        /// Payment has been cancelled by the creator
        PaymentCancelled(T::AccountId, T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The selected payment does not exist
        InvalidPayment,
        /// The selected payment cannot be released
        PaymentAlreadyReleased,
        /// The selected payment already exists and is in process
        PaymentAlreadyInProcess,
        /// Action permitted only for whitelisted users
        InvalidAction,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// This allows any user to create a new payment, that releases only to specified recipient
        /// The only action is to store the details of this payment in storage and reserve
        /// the specified amount.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create(
            origin: OriginFor<T>,
            recipient: T::AccountId,
            asset: AssetIdOf<T>,
            amount: BalanceOf<T>,
            incentive_amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            <Self as PaymentHandler<T::AccountId, AssetIdOf<T>, BalanceOf<T>>>::create_payment(
                who,
                recipient,
                asset,
                amount,
                incentive_amount,
            )?;
            Ok(().into())
        }

        /// Release any created payment, this will transfer the reserved amount from the
        /// creator of the payment to the assigned recipient
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn release(origin: OriginFor<T>, to: T::AccountId) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            <Self as PaymentHandler<T::AccountId, AssetIdOf<T>, BalanceOf<T>>>::release_payment(
                who, to,
            )?;
            Ok(().into())
        }

        /// Cancel a payment in created state, this will release the reserved back to
        /// creator of the payment. This extrinsic can only be called by the recipient
        /// of the payment
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn cancel(origin: OriginFor<T>, creator: T::AccountId) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            <Self as PaymentHandler<T::AccountId, AssetIdOf<T>, BalanceOf<T>>>::cancel_payment(
                creator, who, // the caller must be the provider, creator cannot cancel
            )?;
            Ok(().into())
        }

        /// Allow admins to set state of a payment
        /// This extrinsic is used to resolve disputes between the creator and
        /// recipient of the payment.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn resolve(
            origin: OriginFor<T>,
            from: T::AccountId,
            recipient: T::AccountId,
            new_state: PaymentState,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // ensure the caller is part of the whitelist
            ensure!(T::JudgeWhitelist::contains(&who), Error::<T>::InvalidAction);
            // try to update the payment to new state
            use PaymentState::*;
            match new_state {
                Cancelled => {
                    <Self as PaymentHandler<T::AccountId, AssetIdOf<T>, BalanceOf<T>>>::cancel_payment(
                        from, recipient,
                    )
                }
                Released => <Self as PaymentHandler<
                    T::AccountId,
                    AssetIdOf<T>,
                    BalanceOf<T>,
                >>::release_payment(from, recipient),
                Created | NeedsReview => Err(Error::<T>::InvalidAction.into()),
            }?;
            Ok(().into())
        }
    }

    impl<T: Config> PaymentHandler<T::AccountId, AssetIdOf<T>, BalanceOf<T>> for Pallet<T> {
        fn create_payment(
            from: T::AccountId,
            recipient: T::AccountId,
            asset: AssetIdOf<T>,
            amount: BalanceOf<T>,
            incentive_amount: BalanceOf<T>,
        ) -> DispatchResult {
            Payment::<T>::try_mutate(from.clone(), recipient, |maybe_payment| -> DispatchResult {
                let new_payment = Some(PaymentDetail {
                    asset,
                    amount,
                    incentive_amount,
                    state: PaymentState::Created,
                });
                match maybe_payment {
                    Some(x) => {
                        // do not overwrite an in-process payment!
                        // ensure the payment is not in created state, it should
                        // be in released/cancelled, in which case it can be overwritten
                        ensure!(
                            x.state != PaymentState::Created,
                            Error::<T>::PaymentAlreadyInProcess
                        );
                        // reserve the (payment amount + incentive amount) from the payment creator
                        T::Asset::reserve(asset, &from, amount + incentive_amount)?;
                        *maybe_payment = new_payment
                    }
                    None => {
                        // reserve the (payment amount + incentive amount)from the payment creator
                        T::Asset::reserve(asset, &from, amount + incentive_amount)?;
                        *maybe_payment = new_payment
                    }
                }
                Ok(())
            })
        }

        fn release_payment(from: T::AccountId, to: T::AccountId) -> DispatchResult {
            use PaymentState::*;
            // add the payment detail to storage
            Payment::<T>::try_mutate(
                from.clone(),
                to.clone(),
                |maybe_payment| -> DispatchResult {
                    let payment = maybe_payment.as_mut().ok_or(Error::<T>::InvalidPayment)?;
                    // ensure the payment is in created state
                    ensure!(payment.state == Created, Error::<T>::PaymentAlreadyReleased);
                    // unreserve the (payment amount + incentive amount) from the owner account.
                    // Shouldn't fail for payments created successfully, if user manages to unreserve assets
                    // somehow and be left without enough balance we set the payment to a "corrupted" state.
                    T::Asset::unreserve(payment.asset, &from, payment.amount + payment.incentive_amount);
                    // transfer only the payment amount to the recipient 
                    match T::Asset::transfer(payment.asset, &from, &to, payment.amount) {
                        Ok(_) => payment.state = PaymentState::Released,
                        Err(_) => payment.state = PaymentState::NeedsReview,
                    }
                    Ok(())
                },
            )?;

            Self::deposit_event(Event::PaymentReleased(from, to));
            Ok(())
        }

        fn cancel_payment(from: T::AccountId, to: T::AccountId) -> DispatchResult {
            // add the payment detail to storage
            Payment::<T>::try_mutate(
                from.clone(),
                to.clone(),
                |maybe_payment| -> DispatchResult {
                    let payment = maybe_payment.take().ok_or(Error::<T>::InvalidPayment)?;
                    // ensure the payment is in created state
                    ensure!(
                        payment.state == PaymentState::Created,
                        Error::<T>::PaymentAlreadyReleased
                    );
                    // unreserve the (payment amount + incentive amount) from the owner account
                    T::Asset::unreserve(payment.asset, &from, payment.amount + payment.incentive_amount);
                    *maybe_payment = Some(PaymentDetail {
                        state: PaymentState::Cancelled,
                        ..payment
                    });
                    Ok(())
                },
            )?;
            Self::deposit_event(Event::PaymentReleased(from, to));
            Ok(())
        }

        fn get_payment_details(
            from: T::AccountId,
            to: T::AccountId,
        ) -> Option<PaymentDetail<AssetIdOf<T>, BalanceOf<T>>> {
            Payment::<T>::get(from, to)
        }
    }
}
