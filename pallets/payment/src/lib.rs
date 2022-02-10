#![allow(clippy::unused_unit, unused_qualifications, missing_debug_implementations)]
#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod types;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
	pub use crate::{
		types::{DisputeResolver, FeeHandler, PaymentDetail, PaymentHandler, PaymentState},
		weights::WeightInfo,
	};
	use frame_support::{
		dispatch::DispatchResultWithPostInfo, fail, pallet_prelude::*, require_transactional,
		transactional,
	};
	use frame_system::pallet_prelude::*;
	use orml_traits::{MultiCurrency, MultiReservableCurrency};
	use sp_runtime::{traits::CheckedAdd, Percent};
	use sp_std::vec::Vec;

	pub type BalanceOf<T> =
		<<T as Config>::Asset as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
	pub type AssetIdOf<T> =
		<<T as Config>::Asset as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;
	pub type BoundedDataOf<T> = BoundedVec<u8, <T as Config>::MaxRemarkLength>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// the type of assets this pallet can hold in payment
		type Asset: MultiReservableCurrency<Self::AccountId>;
		/// Dispute resolution account
		type DisputeResolver: DisputeResolver<Self::AccountId>;
		/// Fee handler trait
		type FeeHandler: FeeHandler<Self>;
		/// Incentive percentage - amount witheld from sender
		#[pallet::constant]
		type IncentivePercentage: Get<Percent>;
		/// Maximum permitted size of `Remark`
		#[pallet::constant]
		type MaxRemarkLength: Get<u32>;
		/// Buffer period - number of blocks to wait before user can claim canceled payment
		#[pallet::constant]
		type CancelBufferBlockLength: Get<Self::BlockNumber>;
		//// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
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
		PaymentDetail<T>,
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
		/// the payment creator has created a refund request
		PaymentCreatorRequestedRefund {
			from: T::AccountId,
			to: T::AccountId,
			expiry: T::BlockNumber,
		},
		/// the refund request from creator was disputed by recipient
		PaymentRefundDisputed { from: T::AccountId, to: T::AccountId },
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
		/// Unexpeted math error
		MathError,
		/// Payment request has not been created
		RefundNotRequested,
		/// Dispute period has not passed
		DisputePeriodNotPassed,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// This allows any user to create a new payment, that releases only to specified recipient
		/// The only action is to store the details of this payment in storage and reserve
		/// the specified amount.
		#[transactional]
		#[pallet::weight(T::WeightInfo::pay())]
		pub fn pay(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			asset: AssetIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			<Self as PaymentHandler<T>>::create_payment(who, recipient, asset, amount, None)?;
			Ok(().into())
		}

		/// This allows any user to create a new payment with the option to add a remark, this remark
		/// can then be used to run custom logic and trigger alternate payment flows.
		/// the specified amount.
		#[transactional]
		#[pallet::weight(T::WeightInfo::pay_with_remark())]
		pub fn pay_with_remark(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			asset: AssetIdOf<T>,
			amount: BalanceOf<T>,
			remark: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			// ensure remark is not too large
			let bounded_remark: BoundedDataOf<T> =
				remark.try_into().map_err(|_| Error::<T>::RemarkTooLarge)?;

			<Self as PaymentHandler<T>>::create_payment(
				who,
				recipient,
				asset,
				amount,
				Some(bounded_remark),
			)?;
			Ok(().into())
		}

		/// Release any created payment, this will transfer the reserved amount from the
		/// creator of the payment to the assigned recipient
		#[transactional]
		#[pallet::weight(T::WeightInfo::release())]
		pub fn release(origin: OriginFor<T>, to: T::AccountId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			<Self as PaymentHandler<T>>::release_payment(who, to)?;
			Ok(().into())
		}

		/// Cancel a payment in created state, this will release the reserved back to
		/// creator of the payment. This extrinsic can only be called by the recipient
		/// of the payment
		#[transactional]
		#[pallet::weight(T::WeightInfo::cancel())]
		pub fn cancel(origin: OriginFor<T>, creator: T::AccountId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			if let Some(payment) = Payment::<T>::get(creator.clone(), who.clone()) {
				match payment.state {
					// call settle payment with recipient_share=0, this refunds the sender
					PaymentState::Created => {
						<Self as PaymentHandler<T>>::settle_payment(
							creator.clone(),
							who.clone(),
							Percent::from_percent(0),
						)?;
						Self::deposit_event(Event::PaymentCancelled { from: creator, to: who });
					},
					_ => fail!(Error::<T>::InvalidAction),
				}
			} else {
				fail!(Error::<T>::InvalidPayment);
			}
			Ok(().into())
		}

		/// Allow admins to set state of a payment
		/// This extrinsic is used to resolve disputes between the creator and
		/// recipient of the payment. This extrinsic allows the assigned judge to cancel the payment
		#[transactional]
		#[pallet::weight(T::WeightInfo::resolve_cancel_payment())]
		pub fn resolve_cancel_payment(
			origin: OriginFor<T>,
			from: T::AccountId,
			recipient: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			// ensure the caller is the assigned resolver
			if let Some(payment) = Payment::<T>::get(from.clone(), recipient.clone()) {
				ensure!(who == payment.resolver_account, Error::<T>::InvalidAction)
			}
			// try to update the payment to new state
			<Self as PaymentHandler<T>>::settle_payment(from, recipient, Percent::from_percent(0))?;
			Ok(().into())
		}

		/// Allow admins to set state of a payment
		/// This extrinsic is used to resolve disputes between the creator and
		/// recipient of the payment. This extrinsic allows the assigned judge to send the payment to recipient
		#[transactional]
		#[pallet::weight(T::WeightInfo::resolve_release_payment())]
		pub fn resolve_release_payment(
			origin: OriginFor<T>,
			from: T::AccountId,
			recipient: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			// ensure the caller is the assigned resolver
			if let Some(payment) = Payment::<T>::get(from.clone(), recipient.clone()) {
				ensure!(who == payment.resolver_account, Error::<T>::InvalidAction)
			}
			// try to update the payment to new state
			<Self as PaymentHandler<T>>::release_payment(from, recipient)?;
			Ok(().into())
		}

		/// Allow payment creator to set payment to NeedsReview
		/// This extrinsic is used to mark the payment as disputed so the assigned judge can tigger a resolution
		/// and that the funds are no longer locked.
		#[transactional]
		#[pallet::weight(T::WeightInfo::request_refund())]
		pub fn request_refund(
			origin: OriginFor<T>,
			recipient: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Payment::<T>::try_mutate(
				who.clone(),
				recipient.clone(),
				|maybe_payment| -> DispatchResult {
					// ensure the payment exists
					let payment = maybe_payment.as_mut().ok_or(Error::<T>::InvalidPayment)?;
					// ensure the payment is not in needsreview state
					ensure!(
						payment.state != PaymentState::NeedsReview,
						Error::<T>::PaymentNeedsReview
					);

					// set the payment to requested refund
					let current_block = frame_system::Pallet::<T>::block_number();
					let can_cancel_block = current_block
						.checked_add(&T::CancelBufferBlockLength::get())
						.ok_or(Error::<T>::MathError)?;
					payment.state = PaymentState::RefundRequested(can_cancel_block);

					Self::deposit_event(Event::PaymentCreatorRequestedRefund {
						from: who,
						to: recipient,
						expiry: can_cancel_block,
					});

					Ok(())
				},
			)?;

			Ok(().into())
		}

		/// Allow payment creator to claim the refund if the payment recipent has not disputed
		/// After the payment creator has `request_refund` can then call this extrinsic to
		/// cancel the payment and receive the reserved amount to the account if the dispute period
		/// has passed.
		#[transactional]
		#[pallet::weight(T::WeightInfo::claim_refund())]
		pub fn claim_refund(
			origin: OriginFor<T>,
			recipient: T::AccountId,
		) -> DispatchResultWithPostInfo {
			use PaymentState::*;
			let who = ensure_signed(origin)?;

			if let Some(payment) = Payment::<T>::get(who.clone(), recipient.clone()) {
				match payment.state {
					NeedsReview => fail!(Error::<T>::PaymentNeedsReview),
					Created => fail!(Error::<T>::RefundNotRequested),
					RefundRequested(cancel_block) => {
						let current_block = frame_system::Pallet::<T>::block_number();
						// ensure the dispute period has passed
						ensure!(current_block > cancel_block, Error::<T>::DisputePeriodNotPassed);
						// cancel the payment and refund the creator
						<Self as PaymentHandler<T>>::settle_payment(
							who.clone(),
							recipient.clone(),
							Percent::from_percent(0),
						)?;
						Self::deposit_event(Event::PaymentCancelled { from: who, to: recipient });
					},
				}
			} else {
				fail!(Error::<T>::InvalidPayment);
			}

			Ok(().into())
		}

		/// Allow payment recipient to dispute the refund request from the payment creator
		/// This does not cancel the request, instead sends the payment to a NeedsReview state
		/// The assigned resolver account can then change the state of the payment after review.
		#[transactional]
		#[pallet::weight(T::WeightInfo::dispute_refund())]
		pub fn dispute_refund(
			origin: OriginFor<T>,
			creator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			use PaymentState::*;
			let who = ensure_signed(origin)?;

			Payment::<T>::try_mutate(
				creator.clone(),
				who.clone(), // should be called by the payment recipient
				|maybe_payment| -> DispatchResult {
					// ensure the payment exists
					let payment = maybe_payment.as_mut().ok_or(Error::<T>::InvalidPayment)?;
					// ensure the payment is in Requested Refund state
					match payment.state {
						RefundRequested(_) => {
							payment.state = PaymentState::NeedsReview;

							Self::deposit_event(Event::PaymentRefundDisputed {
								from: creator,
								to: who,
							});
						},
						_ => fail!(Error::<T>::InvalidAction),
					}

					Ok(())
				},
			)?;

			Ok(().into())
		}
	}

	impl<T: Config> PaymentHandler<T> for Pallet<T> {
		/// The function will create a new payment. When a new payment is created, an amount + incentive
		/// is reserved from the payment creator. The incentive amount is reserved in the creators account.
		/// The amount is transferred to the payment recipent but kept in reserved state. Only when the release action
		/// is triggered the amount is released to the recipent and incentive released to creator.
		#[require_transactional]
		fn create_payment(
			from: T::AccountId,
			recipient: T::AccountId,
			asset: AssetIdOf<T>,
			amount: BalanceOf<T>,
			remark: Option<BoundedDataOf<T>>,
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
		#[require_transactional]
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

					match &payment.fee_detail {
						Some((fee_recipient_account, fee_amount)) => {
							// unreserve the incentive amount + fees back to the creator
							T::Asset::unreserve(
								payment.asset,
								&from,
								payment.incentive_amount + *fee_amount,
							);
							// unreserve the amount to the recipent
							T::Asset::unreserve(payment.asset, &to, payment.amount);
							// transfer fee amount to marketplace
							T::Asset::transfer(
								payment.asset,
								&from,                  // fee is paid by payment creator
								&fee_recipient_account, // account of fee recipient
								*fee_amount,            // amount of fee
							)?;
						},
						None => {
							// unreserve the incentive amount back to the creator
							T::Asset::unreserve(payment.asset, &from, payment.incentive_amount);
							// unreserve the amount to the recipent
							T::Asset::unreserve(payment.asset, &to, payment.amount);
						},
					}

					// clear payment data from storage
					*maybe_payment = None;
					Ok(())
				},
			)?;
			Self::deposit_event(Event::PaymentReleased { from, to });
			Ok(())
		}

		/// This function allows the caller to settle the payment by specifying a recipient_share
		/// For cancelling a payment, recipient_share = 0
		/// For releasing a payment, recipient_share = 100
		#[require_transactional]
		fn settle_payment(
			from: T::AccountId,
			to: T::AccountId,
			recipient_share: Percent,
		) -> DispatchResult {
			Payment::<T>::try_mutate(
				from.clone(),
				to.clone(),
				|maybe_payment| -> DispatchResult {
					let payment = maybe_payment.take().ok_or(Error::<T>::InvalidPayment)?;

					// unreserve the incentive amount and fees from the owner account
					match payment.fee_detail {
						Some((_, fee_amount)) => {
							T::Asset::unreserve(
								payment.asset,
								&from,
								payment.incentive_amount + fee_amount,
							);
						},
						None => {
							T::Asset::unreserve(payment.asset, &from, payment.incentive_amount);
						},
					};

					// Unreserve the transfer amount
					T::Asset::unreserve(payment.asset, &to, payment.amount);

					let amount_to_recipient = recipient_share * payment.amount;
					let amount_to_sender = payment.amount - amount_to_recipient;
					// send share to recipient
					T::Asset::transfer(payment.asset, &to, &from, amount_to_sender)?;

					Ok(())
				},
			)?;
			Ok(())
		}

		fn get_payment_details(from: T::AccountId, to: T::AccountId) -> Option<PaymentDetail<T>> {
			Payment::<T>::get(from, to)
		}
	}
}
