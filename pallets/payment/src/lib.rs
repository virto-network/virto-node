#![allow(clippy::unused_unit, unused_qualifications, missing_debug_implementations)]
#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;

#[frame_support::pallet]
pub mod pallet {
	pub use crate::types::{
		DisputeResolver, FeeHandler, PaymentDetail, PaymentHandler, PaymentState,
	};
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use orml_traits::{MultiCurrency, MultiReservableCurrency};
	use sp_runtime::Percent;
	use sp_std::vec::Vec;

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
		/// Fee handler trait
		type FeeHandler: FeeHandler<AssetIdOf<Self>, BalanceOf<Self>, Self::AccountId>;
		/// Incentive percentage - amount witheld from sender
		#[pallet::constant]
		type IncentivePercentage: Get<Percent>;
		/// Incentive percentage - amount witheld from sender
		#[pallet::constant]
		type MaxRemarkLength: Get<u32>;
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
		PaymentCreated { from: T::AccountId, asset: AssetIdOf<T>, amount: BalanceOf<T> },
		/// Payment amount released to the recipient
		PaymentReleased { from: T::AccountId, to: T::AccountId },
		/// Payment has been cancelled by the creator
		PaymentCancelled { from: T::AccountId, to: T::AccountId },
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
		/// Remark size is larger than permitted
		RemarkTooLarge,
		/// Payment is in review state and cannot be modified
		PaymentNeedsReview,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// This allows any user to create a new payment, that releases only to specified recipient
		/// The only action is to store the details of this payment in storage and reserve
		/// the specified amount.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn pay(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			asset: AssetIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			<Self as PaymentHandler<T::AccountId, AssetIdOf<T>, BalanceOf<T>>>::create_payment(
				who, recipient, asset, amount, None,
			)?;
			Ok(().into())
		}

		/// This allows any user to create a new payment with the option to add a remark, this remark
		/// can then be used to run custom logic and trigger alternate payment flows.
		/// the specified amount.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn pay_with_remark(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			asset: AssetIdOf<T>,
			amount: BalanceOf<T>,
			remark: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			// ensure remark is not too large
			ensure!(
				remark.len() <= T::MaxRemarkLength::get().try_into().unwrap(),
				Error::<T>::RemarkTooLarge
			);

			<Self as PaymentHandler<T::AccountId, AssetIdOf<T>, BalanceOf<T>>>::create_payment(
				who,
				recipient,
				asset,
				amount,
				Some(remark),
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
			remark: Option<Vec<u8>>,
		) -> DispatchResult {
			Payment::<T>::try_mutate(
				from.clone(),
				recipient.clone(),
				|maybe_payment| -> DispatchResult {
					// ensure a payment is not already in process
					if maybe_payment.is_some() {
						// do not overwrite an in-process payment!
						// ensure the payment is not in created/needsreview state, it should
						// be in released/cancelled, in which case it can be overwritten
						let current_state = maybe_payment.clone().unwrap().state;
						ensure!(
							current_state != PaymentState::Created,
							Error::<T>::PaymentAlreadyInProcess
						);
						ensure!(
							current_state != PaymentState::NeedsReview,
							Error::<T>::PaymentNeedsReview
						);
					}
					// Calculate incentive amount - this is to insentivise the user to release
					// the funds once a transaction has been completed
					let incentive_amount = T::IncentivePercentage::get() * amount;

					let mut new_payment = PaymentDetail {
						asset,
						amount,
						incentive_amount,
						state: PaymentState::Created,
						resolver_account: T::DisputeResolver::get_origin(),
						fee_detail: None,
						remark,
					};

					// Calculate fee amount - this will be implemented based on the custom
					// implementation of the marketplace
					let (fee_recipient, fee_percent) =
						T::FeeHandler::apply_fees(&from, &recipient, &new_payment);
					let fee_amount = fee_percent * amount;
					new_payment.fee_detail = Some((fee_recipient, fee_amount));

					// reserve the incentive amount from the payment creator
					T::Asset::reserve(asset, &from, incentive_amount + fee_amount)?;
					// transfer amount to recipient
					T::Asset::transfer(asset, &from, &recipient, amount)?;
					// reserved the amount in the recipient account
					T::Asset::reserve(asset, &recipient, amount)?;

					*maybe_payment = Some(new_payment);

					Self::deposit_event(Event::PaymentCreated { from, asset, amount });
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
					let (fee_recipient_account, fee_amount) =
						payment.fee_detail.clone().unwrap_or_default();
					// unreserve the incentive amount back to the creator
					T::Asset::unreserve(
						payment.asset,
						&from,
						payment.incentive_amount + fee_amount,
					);
					// unreserve the amount to the recipent
					T::Asset::unreserve(payment.asset, &to, payment.amount);
					// transfer fee amount to marketplace
					T::Asset::transfer(
						payment.asset,
						&from,                  // fee is paid by payment creator
						&fee_recipient_account, // account of fee recipient
						fee_amount,             // amount of fee
					)?;
					payment.state = PaymentState::Released;

					Ok(())
				},
			)?;

			Self::deposit_event(Event::PaymentReleased { from, to });
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
					T::Asset::unreserve(
						payment.asset,
						&from,
						payment.incentive_amount + payment.fee_detail.clone().unwrap_or_default().1,
					);
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
			Self::deposit_event(Event::PaymentCancelled { from, to });
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
