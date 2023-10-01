#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::pallet_prelude::BlockNumberFor;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use sp_io::hashing::blake2_256;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub use codec::{Decode, Encode, MaxEncodedLen};

use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	ensure, fail,
	traits::{
		fungibles::{
			hold::Mutate as FunHoldMutate, Balanced as FunBalanced, Inspect as FunInspect, Mutate as FunMutate,
		},
		schedule::{v3::Named as ScheduleNamed, DispatchTime},
		tokens::{
			fungibles::Inspect as FunsInspect,
			Fortitude::Polite,
			Precision::Exact,
			Preservation::{Expendable, Preserve},
		},
		Bounded, CallerTrait, OriginTrait, QueryPreimage, StorePreimage,
	},
};

pub mod weights;
use sp_runtime::{
	traits::{CheckedAdd, Dispatchable, One, StaticLookup, Zero},
	DispatchError, DispatchResult, Percent, Saturating,
};
pub use weights::*;

pub mod types;
pub use types::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, PalletId};
	use frame_system::pallet_prelude::*;

	use sp_runtime::{traits::Get, Percent};
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The caller origin, overarching type of all pallets origins.
		type PalletsOrigin: From<frame_system::RawOrigin<Self::AccountId>>
			+ CallerTrait<Self::AccountId>
			+ MaxEncodedLen;

		/// The aggregated call type.
		type RuntimeCall: Parameter
			+ Dispatchable<RuntimeOrigin = Self::RuntimeOrigin, PostInfo = PostDispatchInfo>
			+ GetDispatchInfo
			+ From<Call<Self>>;

		/// Currency type that this works on.
		type Assets: FunInspect<Self::AccountId, Balance = Self::AssetsBalance>
			+ FunMutate<Self::AccountId>
			+ FunBalanced<Self::AccountId>
			+ FunHoldMutate<Self::AccountId, Reason = Self::RuntimeHoldReason>
			+ FunsInspect<Self::AccountId>;

		/// Just the `Currency::Balance` type; we have this item to allow us to
		/// constrain it to `From<u64>`.
		type AssetsBalance: sp_runtime::traits::AtLeast32BitUnsigned
			+ codec::FullCodec
			+ Copy
			+ MaybeSerializeDeserialize
			+ sp_std::fmt::Debug
			+ Default
			+ From<u64>
			+ TypeInfo
			+ MaxEncodedLen;

		type FeeHandler: FeeHandler<Self>;

		type DisputeResolver: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

		type PaymentId: Member
			+ Parameter
			+ Copy
			+ Clone
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ Saturating
			+ One
			+ Zero;

		type Scheduler: ScheduleNamed<BlockNumberFor<Self>, CallOf<Self>, Self::PalletsOrigin>;
		/// The preimage provider with which we look up call hashes to get the
		/// call.
		type Preimages: QueryPreimage + StorePreimage;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[pallet::constant]
		type IncentivePercentage: Get<Percent>;

		/// The overarching hold reason.
		type RuntimeHoldReason: From<HoldReason>;

		#[pallet::constant]
		type MaxRemarkLength: Get<u32>;

		#[pallet::constant]
		type MaxFees: Get<u32>;

		#[pallet::constant]
		type MaxDiscounts: Get<u32>;

		/// Buffer period - number of blocks to wait before user can claim
		/// canceled payment
		#[pallet::constant]
		type CancelBufferBlockLength: Get<BlockNumberFor<Self>>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn payment)]
	/// Payments created by a user, this method of storageDoubleMap is chosen
	/// since there is no usecase for listing payments by provider/currency. The
	/// payment will only be referenced by the creator in any transaction of
	/// interest. The storage map keys are the creator and the recipient, this
	/// also ensures that for any (sender,recipient) combo, only a single
	/// payment is active. The history of payment is not stored.
	pub type Payment<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, T::AccountId>,
			NMapKey<Blake2_128Concat, T::AccountId>,
			NMapKey<Blake2_128Concat, T::PaymentId>,
		),
		PaymentDetail<T>,
		ResultQuery<Error<T>::NonExistentStorageValue>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn last_id)]
	pub type LastId<T: Config> = StorageValue<_, T::PaymentId, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new payment has been created
		PaymentCreated {
			sender: T::AccountId,
			beneficiary: T::AccountId,
			asset: AssetIdOf<T>,
			amount: BalanceOf<T>,
			remark: Option<BoundedDataOf<T>>,
		},
		/// Payment amount released to the recipient
		PaymentReleased {
			sender: T::AccountId,
			beneficiary: T::AccountId,
		},
		/// Payment has been cancelled by the creator
		PaymentCancelled {
			sender: T::AccountId,
			beneficiary: T::AccountId,
		},
		/// A payment that NeedsReview has been resolved by Judge
		PaymentResolved {
			sender: T::AccountId,
			beneficiary: T::AccountId,
			recipient_share: Percent,
		},
		/// the payment creator has created a refund request
		PaymentCreatorRequestedRefund {
			sender: T::AccountId,
			beneficiary: T::AccountId,
			expiry: BlockNumberFor<T>,
		},
		/// the payment was refunded
		PaymentRefunded {
			sender: T::AccountId,
			beneficiary: T::AccountId,
		},
		/// the refund request from creator was disputed by recipient
		PaymentRefundDisputed {
			sender: T::AccountId,
			beneficiary: T::AccountId,
		},
		/// Payment request was created by recipient
		PaymentRequestCreated {
			sender: T::AccountId,
			beneficiary: T::AccountId,
		},
		/// Payment request was completed by sender
		PaymentRequestCompleted {
			sender: T::AccountId,
			beneficiary: T::AccountId,
		},
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
		/// Payment is in review state and cannot be modified
		PaymentNeedsReview,
		/// Unexpeted math error
		MathError,
		/// Payment request has not been created
		RefundNotRequested,
		/// Dispute period has not passed
		DisputePeriodNotPassed,
		/// The automatic cancelation queue cannot accept
		RefundQueueFull,
		/// Release was not possible
		ReleaseFailed,
		/// Transfer failed
		TransferFailed,

		NonExistentStorageValue,
	}

	#[pallet::composite_enum]
	pub enum HoldReason {
		#[codec(index = 0)]
		TransferPayment,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// This allows any user to create a new payment, that releases only to
		/// specified recipient The only action is to store the details of this
		/// payment in storage and reserve the specified amount. User also has
		/// the option to add a remark, this remark can then be used to run
		/// custom logic and trigger alternate payment flows. the specified
		/// amount.
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn pay(
			origin: OriginFor<T>,
			beneficiary: AccountIdLookupOf<T>,
			asset: AssetIdOf<T>,
			#[pallet::compact] amount: BalanceOf<T>,
			remark: Option<BoundedDataOf<T>>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let beneficiary = T::Lookup::lookup(beneficiary)?;

			let last_id = LastId::<T>::get().unwrap_or(Zero::zero());

			let payment_id: T::PaymentId = last_id.saturating_add(One::one());

			// create PaymentDetail and add to storage
			let payment_detail = Self::create_payment(
				&sender,
				&beneficiary,
				asset.clone(),
				payment_id,
				amount,
				PaymentState::Created,
				T::IncentivePercentage::get(),
				remark.as_ref().map(|x| x.as_slice()),
			)?;

			// reserve funds for payment
			Self::reserve_payment_amount(&sender, &beneficiary, payment_detail)?;
			// emit paymentcreated event
			Self::deposit_event(Event::PaymentCreated {
				sender,
				beneficiary,
				asset,
				amount,
				remark,
			});
			Ok(().into())
		}

		/// Release any created payment, this will transfer the reserved amount
		/// from the creator of the payment to the assigned recipient
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn release(
			origin: OriginFor<T>,
			beneficiary: AccountIdLookupOf<T>,
			payment_id: T::PaymentId,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let beneficiary = T::Lookup::lookup(beneficiary)?;

			// ensure the payment is in Created state
			let payment =
				Payment::<T>::get((&sender, &beneficiary, &payment_id)).map_err(|_| Error::<T>::InvalidPayment)?;
			ensure!(payment.state == PaymentState::Created, Error::<T>::InvalidAction);

			Self::settle_payment(&sender, &beneficiary, &payment_id, None)?;

			Self::deposit_event(Event::PaymentReleased { sender, beneficiary });
			Ok(().into())
		}

		/// Cancel a payment in created state, this will release the reserved
		/// back to creator of the payment. This extrinsic can only be called by
		/// the recipient of the payment
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn cancel(
			origin: OriginFor<T>,
			sender: AccountIdLookupOf<T>,
			payment_id: T::PaymentId,
		) -> DispatchResultWithPostInfo {
			let beneficiary = ensure_signed(origin)?;
			let sender = T::Lookup::lookup(sender)?;

			let payment =
				Payment::<T>::get((&sender, &beneficiary, &payment_id)).map_err(|_| Error::<T>::InvalidPayment)?;

			match payment.state {
				PaymentState::Created => {
					Self::cancel_payment(&sender, &beneficiary, payment)?;
					Self::deposit_event(Event::PaymentCancelled {
						sender: sender.clone(),
						beneficiary: beneficiary.clone(),
					});
				}
				PaymentState::RefundRequested { cancel_block: _ } => {
					Self::cancel_payment(&sender, &beneficiary, payment)?;
					Self::deposit_event(Event::PaymentRefunded {
						sender: sender.clone(),
						beneficiary: beneficiary.clone(),
					});
				}
				_ => fail!(Error::<T>::InvalidAction),
			}

			Payment::<T>::remove((&sender, &beneficiary, &payment_id));

			Ok(().into())
		}

		/// Allow the creator of a payment to initiate a refund that will return
		/// the funds after a configured amount of time that the reveiver has to
		/// react and opose the request
		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn request_refund(
			origin: OriginFor<T>,
			beneficiary: AccountIdLookupOf<T>,
			payment_id: T::PaymentId,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin.clone())?;
			let beneficiary_account = T::Lookup::lookup(beneficiary)?;

			Payment::<T>::try_mutate(
				(sender.clone(), beneficiary_account.clone(), payment_id),
				|maybe_payment| -> Result<(), sp_runtime::DispatchError> {
					// ensure the payment exists
					let payment = maybe_payment.as_mut().map_err(|_| Error::<T>::InvalidPayment)?;
					// refunds only possible for payments in created state
					ensure!(payment.state == PaymentState::Created, Error::<T>::InvalidAction);

					// set the payment to requested refund
					let current_block = frame_system::Pallet::<T>::block_number();
					let cancel_block = current_block
						.checked_add(&T::CancelBufferBlockLength::get())
						.ok_or(Error::<T>::MathError)?;

					let sender_unlookup = T::Lookup::unlookup(sender.clone());

					let cancel_call = <T as Config>::RuntimeCall::from(pallet::Call::<T>::cancel {
						sender: sender_unlookup,
						payment_id,
					});

					let _ = T::Scheduler::schedule_named(
						("payment", payment_id).using_encoded(blake2_256),
						DispatchTime::At(cancel_block),
						None,
						63,
						frame_system::RawOrigin::Signed(beneficiary_account.clone()).into(),
						T::Preimages::bound(cancel_call)?,
					)
					.is_ok();

					payment.state = PaymentState::RefundRequested { cancel_block };

					Self::deposit_event(Event::PaymentCreatorRequestedRefund {
						sender,
						beneficiary: beneficiary_account,
						expiry: cancel_block,
					});

					Ok(())
				},
			)?;

			Ok(().into())
		}

		/// Allow payment beneficiary to dispute the refund request from the
		/// payment creator This does not cancel the request, instead sends the
		/// payment to a NeedsReview state The assigned resolver account can
		/// then change the state of the payment after review.
		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn dispute_refund(
			origin: OriginFor<T>,
			sender: AccountIdLookupOf<T>,
			payment_id: T::PaymentId,
		) -> DispatchResultWithPostInfo {
			use PaymentState::*;
			let beneficiary = ensure_signed(origin)?;
			let sender = T::Lookup::lookup(sender)?;

			Payment::<T>::try_mutate(
				(sender.clone(), beneficiary.clone(), payment_id),
				|maybe_payment| -> Result<(), sp_runtime::DispatchError> {
					// ensure the payment exists
					let payment = maybe_payment.as_mut().map_err(|_| Error::<T>::InvalidPayment)?;
					// ensure the payment is in Requested Refund state
					match payment.state {
						RefundRequested { cancel_block } => {
							ensure!(
								cancel_block > frame_system::Pallet::<T>::block_number(),
								Error::<T>::InvalidAction
							);

							// Hold beneficiary incentive amount to balance the incentives at the time to
							// resolve the dispute
							let reason = &HoldReason::TransferPayment.into();
							T::Assets::hold(payment.asset.clone(), reason, &beneficiary, payment.incentive_amount)?;

							payment.state = PaymentState::NeedsReview;

							let _ =
								T::Scheduler::cancel_named(("payment", payment_id).using_encoded(blake2_256)).is_ok();

							Self::deposit_event(Event::PaymentRefundDisputed { sender, beneficiary });
						}
						_ => fail!(Error::<T>::InvalidAction),
					}

					Ok(())
				},
			)?;

			Ok(().into())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn resolve_dispute(
			origin: OriginFor<T>,
			sender: AccountIdLookupOf<T>,
			beneficiary: AccountIdLookupOf<T>,
			payment_id: T::PaymentId,
			dispute_result: DisputeResult,
		) -> DispatchResultWithPostInfo {
			let dispute_resolver = T::DisputeResolver::ensure_origin(origin)?;

			let sender = T::Lookup::lookup(sender)?;
			let beneficiary = T::Lookup::lookup(beneficiary)?;

			Payment::<T>::try_mutate(
				(sender.clone(), beneficiary.clone(), payment_id),
				|maybe_payment| -> Result<(), sp_runtime::DispatchError> {
					// ensure the payment exists
					let payment = maybe_payment.as_mut().map_err(|_| Error::<T>::InvalidPayment)?;
					// ensure the payment is in Requested Refund state
					match payment.state {
						PaymentState::NeedsReview => {
							payment.state = PaymentState::Finished;

							let dispute = DisputeResultWithResolver {
								dispute_result,
								dispute_resolver,
							};

							let _ = Self::settle_payment(&sender, &beneficiary, &payment_id, Some(dispute))?;

							Self::deposit_event(Event::PaymentRefundDisputed { sender, beneficiary });
						}
						_ => fail!(Error::<T>::InvalidAction),
					}

					Ok(())
				},
			)?;

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// The function will create a new payment. The fee and incentive
	/// amounts will be calculated and the `PaymentDetail` will be added to
	/// storage.
	#[allow(clippy::too_many_arguments)]
	fn create_payment(
		sender: &T::AccountId,
		beneficiary: &T::AccountId,
		asset: AssetIdOf<T>,
		payment_id: T::PaymentId,
		amount: BalanceOf<T>,
		payment_state: PaymentState<BlockNumberFor<T>>,
		incentive_percentage: Percent,
		remark: Option<&[u8]>,
	) -> Result<PaymentDetail<T>, sp_runtime::DispatchError> {
		Payment::<T>::try_mutate(
			(sender, beneficiary, payment_id),
			|maybe_payment| -> Result<PaymentDetail<T>, sp_runtime::DispatchError> {
				if let Ok(payment) = maybe_payment {
					ensure!(
						payment.state == PaymentState::PaymentRequested,
						Error::<T>::PaymentAlreadyInProcess
					);
				}

				let incentive_amount = incentive_percentage.mul_floor(amount);

				let fees_details: Fees<T> = T::FeeHandler::apply_fees(&asset, sender, beneficiary, &amount, remark);

				let new_payment = PaymentDetail::<T> {
					asset,
					amount,
					incentive_amount,
					state: payment_state,
					fees_details,
				};

				*maybe_payment = Ok(new_payment.clone());

				Ok(new_payment)
			},
		)
	}

	fn reserve_payment_amount(
		sender: &T::AccountId,
		beneficiary: &T::AccountId,
		payment: PaymentDetail<T>,
	) -> DispatchResult {
		let (_fee_recipients, total_fee_from_sender_mandatory, total_fee_from_sender_optional) =
			payment.fees_details.get_fees_details(true, false)?;

		let total_hold_amount = total_fee_from_sender_mandatory
			.saturating_add(payment.incentive_amount)
			.saturating_add(total_fee_from_sender_optional);
		let reason = &HoldReason::TransferPayment.into();
		T::Assets::hold(payment.asset.clone(), reason, sender, total_hold_amount)?;

		T::Assets::transfer_and_hold(
			payment.asset,
			reason,
			sender,
			beneficiary,
			payment.amount,
			Exact,
			Preserve,
			Polite,
		)?;

		Ok(())
	}

	fn cancel_payment(sender: &T::AccountId, beneficiary: &T::AccountId, payment: PaymentDetail<T>) -> DispatchResult {
		let (_fee_recipients, total_fee_from_sender_mandatory, total_fee_from_sender_optional) =
			payment.fees_details.get_fees_details(true, false)?;

		let total_hold_amount = total_fee_from_sender_mandatory
			.saturating_add(payment.incentive_amount)
			.saturating_add(total_fee_from_sender_optional);
		let reason = &HoldReason::TransferPayment.into();

		T::Assets::release(payment.asset.clone(), reason, sender, total_hold_amount, Exact)
			.map_err(|_| Error::<T>::ReleaseFailed)?;

		T::Assets::release(payment.asset.clone(), reason, beneficiary, payment.amount, Exact)
			.map_err(|_| Error::<T>::ReleaseFailed)?;

		T::Assets::transfer(payment.asset, beneficiary, sender, payment.amount, Expendable)
			.map_err(|_| Error::<T>::TransferFailed)?;

		Ok(())
	}

	fn settle_payment(
		sender: &T::AccountId,
		beneficiary: &T::AccountId,
		payment_id: &T::PaymentId,
		dispute: Option<DisputeResultWithResolver<DisputeResult, T::AccountId>>,
	) -> DispatchResult {
		Payment::<T>::try_mutate((sender, beneficiary, payment_id), |maybe_payment| -> DispatchResult {
			let payment = maybe_payment.as_mut().map_err(|_| Error::<T>::InvalidPayment)?;

			let reason = &HoldReason::TransferPayment.into();
			let is_dispute = dispute.is_some();

			// Release sender fees recipients
			let (fee_sender_recipients, total_sender_fee_amount_mandatory, total_sender_fee_amount_optional) =
				payment.fees_details.get_fees_details(true, is_dispute)?;

			let total_sender_release = total_sender_fee_amount_mandatory
				.saturating_add(payment.incentive_amount)
				.saturating_add(total_sender_fee_amount_optional);

			println!("total_sender_release: {:?}", total_sender_release);

			T::Assets::release(payment.asset.clone(), reason, sender, total_sender_release, Exact)
				.map_err(|_| Error::<T>::ReleaseFailed)?;

			let (
				fee_beneficiary_recipients,
				_total_beneficiary_fee_amount_mandatory,
				_total_beneficiary_fee_amount_optional,
			) = payment.fees_details.get_fees_details(false, is_dispute)?;

			let beneficiary_release_amount = payment.amount.clone();
			if is_dispute {
				beneficiary_release_amount.saturating_add(payment.incentive_amount);
			}
			println!("beneficiary_release_amount: {:?}", beneficiary_release_amount);

			T::Assets::release(
				payment.asset.clone(),
				reason,
				beneficiary,
				beneficiary_release_amount,
				Exact,
			)
			.map_err(|_| Error::<T>::ReleaseFailed)?;

			match dispute {
				Some(dispute) => {
					let dispute_result = &dispute.dispute_result;

					Self::get_and_transfer_fees(sender, payment, fee_sender_recipients, is_dispute)?;

					Self::get_and_transfer_fees(beneficiary, payment, fee_beneficiary_recipients, is_dispute)?;

					let amount_to_beneficiary = dispute_result.percent_beneficiary.mul_floor(payment.amount);
					let amount_to_sender = payment.amount.saturating_sub(amount_to_beneficiary);

					println!("amount_to_beneficiary: {:?}", amount_to_beneficiary);
					println!("amount_to_sender: {:?}", amount_to_sender);

					match dispute_result.in_favor_of {
						Role::Sender => {
							// Beneficiary looses the dispute and has to transfer the incentive_amount to
							// the dispute_resolver.
							T::Assets::transfer(
								payment.asset.clone(),
								beneficiary,
								&dispute.dispute_resolver,
								payment.incentive_amount,
								Expendable,
							)
							.map_err(|_| Error::<T>::TransferFailed)?;

							//
							T::Assets::transfer(
								payment.asset.clone(),
								beneficiary,
								sender,
								amount_to_sender,
								Expendable,
							)
							.map_err(|_| Error::<T>::TransferFailed)?;
						}
						Role::Beneficiary => {
							T::Assets::transfer(
								payment.asset.clone(),
								sender,
								&dispute.dispute_resolver,
								payment.incentive_amount,
								Expendable,
							)
							.map_err(|_| Error::<T>::TransferFailed)?;

							T::Assets::transfer(
								payment.asset.clone(),
								beneficiary,
								sender,
								amount_to_sender,
								Expendable,
							)
							.map_err(|_| Error::<T>::TransferFailed)?;

							T::Assets::release(
								payment.asset.clone(),
								reason,
								beneficiary,
								payment.incentive_amount,
								Exact,
							)
							.map_err(|_| Error::<T>::ReleaseFailed)?;
						}
					}
				}
				None => {
					Self::get_and_transfer_fees(sender, payment, fee_sender_recipients, is_dispute)?;
					Self::get_and_transfer_fees(beneficiary, payment, fee_beneficiary_recipients, is_dispute)?;
				}
			}

			payment.state = PaymentState::Finished;
			*maybe_payment = Ok(payment.clone());

			Ok(())
		})?;
		Ok(())
	}

	fn get_and_transfer_fees(
		account: &T::AccountId,
		payment: &PaymentDetail<T>,
		fee_recipients: Vec<Fee<T>>,
		is_dispute: bool,
	) -> Result<(), sp_runtime::DispatchError> {
		for (recipient_account, fee_amount, mandatory) in fee_recipients.iter() {
			println!(
				"recipient_account: {:?}, fee_amount: {:?} mandatory: {:?}",
				recipient_account, fee_amount, mandatory
			);
			if (is_dispute && *mandatory) || !is_dispute {
				T::Assets::transfer(payment.asset.clone(), account, recipient_account, *fee_amount, Preserve)
					.map_err(|_| Error::<T>::TransferFailed)?;
			}
		}
		Ok(())
	}
}
