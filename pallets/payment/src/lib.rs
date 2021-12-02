#![allow(clippy::unused_unit, unused_qualifications, missing_debug_implementations)]
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
	use sp_runtime::Percent;
	use virto_primitives::{DisputeResolver, PaymentDetail, PaymentHandler, PaymentState};

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
		/// Dispute resolution account
		type DisputeResolver: DisputeResolver<Self::AccountId>;

		#[pallet::constant]
		type IncentivePercentage: Get<Percent>;
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
		PaymentDetail<AssetIdOf<T>, BalanceOf<T>, T::AccountId>,
	>;

	#[pallet::event]
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
			resolver: Option<T::AccountId>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			<Self as PaymentHandler<T::AccountId, AssetIdOf<T>, BalanceOf<T>>>::create_payment(
				who, recipient, asset, amount, resolver,
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
			use PaymentState::*;
			let who = ensure_signed(origin)?;
			// ensure the caller is the assigned resolver
			if let Some(payment) = Payment::<T>::get(from.clone(), recipient.clone()) {
				ensure!(who == payment.resolver_account, Error::<T>::InvalidAction)
			}
			// try to update the payment to new state
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
		/// The function will create a new payment. When a new payment is created, an amount + incentive
		/// is reserved from the payment creator. The incentive amount is reserved in the creators account.
		/// The amount is transferred to the payment recipent but kept in reserved state. Only when the release action
		/// is triggered the amount is released to the recipent and incentive released to creator.
		fn create_payment(
			from: T::AccountId,
			recipient: T::AccountId,
			asset: AssetIdOf<T>,
			amount: BalanceOf<T>,
			resolver: Option<T::AccountId>,
		) -> DispatchResult {
			let resolver_account = match resolver {
				Some(x) => x,
				None => T::DisputeResolver::get_origin(),
			};
			Payment::<T>::try_mutate(
				from.clone(),
				recipient.clone(),
				|maybe_payment| -> DispatchResult {
					let incentive_amount = T::IncentivePercentage::get() * amount;
					let new_payment = Some(PaymentDetail {
						asset,
						amount,
						incentive_amount,
						state: PaymentState::Created,
						resolver_account,
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
							// reserve the incentive amount from the payment creator
							T::Asset::reserve(asset, &from, incentive_amount)?;
							// transfer amount to recipient
							T::Asset::transfer(asset, &from, &recipient, amount)?;
							// reserved the amount in the recipient account
							T::Asset::reserve(asset, &recipient, amount)?;
							*maybe_payment = new_payment
						},
						None => {
							// reserve the incentive amount from the payment creator
							T::Asset::reserve(asset, &from, incentive_amount)?;
							// transfer amount to recipient
							T::Asset::transfer(asset, &from, &recipient, amount)?;
							// reserved the amount in the recipient account
							T::Asset::reserve(asset, &recipient, amount)?;
							*maybe_payment = new_payment
						},
					}
					Ok(())
				},
			)
		}

		/// The function will release an existing payment, a release action will remove the reserve
		/// placed on both the incentive amount and the transfer amount and it will be "released" to the respective account.
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
					// unreserve the incentive amount back to the creator
					T::Asset::unreserve(payment.asset, &from, payment.incentive_amount);
					// unreserve the amount to the recipent
					T::Asset::unreserve(payment.asset, &to, payment.amount);

					payment.state = PaymentState::Released;

					Ok(())
				},
			)?;

			Self::deposit_event(Event::PaymentReleased(from, to));
			Ok(())
		}

		/// This function will allows user to cancel a payment. When cancelling a payment, steps are
		/// - Unreserve the incentive amount
		/// - Unreserve the payment amount
		/// - Transfer amount from recipent to sender
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
					// unreserve the incentive amount from the owner account
					T::Asset::unreserve(payment.asset, &from, payment.incentive_amount);
					T::Asset::unreserve(payment.asset, &to, payment.amount);
					// transfer amount to creator
					match T::Asset::transfer(payment.asset, &to, &from, payment.amount) {
						Ok(_) =>
							*maybe_payment =
								Some(PaymentDetail { state: PaymentState::Cancelled, ..payment }),
						Err(_) =>
							*maybe_payment =
								Some(PaymentDetail { state: PaymentState::NeedsReview, ..payment }),
					}

					Ok(())
				},
			)?;
			Self::deposit_event(Event::PaymentReleased(from, to));
			Ok(())
		}

		fn get_payment_details(
			from: T::AccountId,
			to: T::AccountId,
		) -> Option<PaymentDetail<AssetIdOf<T>, BalanceOf<T>, T::AccountId>> {
			Payment::<T>::get(from, to)
		}
	}
}
