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

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub use codec::{Decode, Encode, MaxEncodedLen};

use frame_support::{
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
	},
};

pub mod weights;
use sp_runtime::{
	traits::{Dispatchable, One, StaticLookup, Zero},
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

		// System level stuff.
		type RuntimeCall: Parameter
			+ Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
			+ From<Call<Self>>
			+ IsType<<Self as frame_system::Config>::RuntimeCall>
			+ From<frame_system::Call<Self>>;

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

		type DisputeResolver: EnsureOrigin<Self::RuntimeOrigin>;

		type PaymentId: Member
			+ Parameter
			+ Copy
			+ Clone
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ Saturating
			+ One
			+ Zero;

		type Scheduler: ScheduleNamed<BlockNumberFor<Self>, CallOf<Self>, PalletsOriginOf<Self>>;

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

			Self::settle_payment(&sender, &beneficiary, &payment_id)?;

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
					Payment::<T>::remove((&sender, &beneficiary, &payment_id));
				}
				_ => fail!(Error::<T>::InvalidAction),
			}

			Ok(().into())
		}

		/// Allow the creator of a payment to initiate a refund that will return
		/// the funds after a configured amount of time that the reveiver has to
		/// react and opose the request
		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn request_refund(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			payment_id: T::PaymentId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Payment::<T>::try_mutate(
				(who.clone(), recipient.clone(), payment_id),
				|maybe_payment| -> DispatchResult {
					// ensure the payment exists
					let payment = maybe_payment.as_mut().ok_or(Error::<T>::InvalidPayment)?;
					// refunds only possible for payments in created state
					ensure!(payment.state == PaymentState::Created, Error::<T>::InvalidAction);

					// set the payment to requested refund
					let current_block = frame_system::Pallet::<T>::block_number();
					let cancel_block = current_block
						.checked_add(&T::CancelBufferBlockLength::get())
						.ok_or(Error::<T>::MathError)?;

					ScheduledTasks::<T>::try_mutate(|task_list| -> DispatchResult {
						task_list
							.try_insert(
								(who.clone(), recipient.clone()),
								ScheduledTask {
									task: Task::Cancel,
									when: cancel_block,
								},
							)
							.map_err(|_| Error::<T>::RefundQueueFull)?;
						Ok(())
					})?;

					payment.state = PaymentState::RefundRequested { cancel_block };

					Self::deposit_event(Event::PaymentCreatorRequestedRefund {
						from: who,
						to: recipient,
						expiry: cancel_block,
					});

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
		let (_fee_recipients, total_fee_from_sender) = &payment.fees_details.get_fees_details_for_sender()?;

		let total_hold_amount = total_fee_from_sender.saturating_add(payment.incentive_amount);
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
		let (_fee_recipients, total_fee_from_sender) = &payment.fees_details.get_fees_details_for_sender()?;
		let total_hold_amount = total_fee_from_sender.saturating_add(payment.incentive_amount);
		let reason = &HoldReason::TransferPayment.into();
		T::Assets::release(payment.asset.clone(), reason, sender, total_hold_amount, Exact)
			.map_err(|_| Error::<T>::ReleaseFailed)?;

		T::Assets::release(payment.asset.clone(), reason, beneficiary, payment.amount, Exact)
			.map_err(|_| Error::<T>::ReleaseFailed)?;

		T::Assets::transfer(payment.asset, beneficiary, sender, payment.amount, Expendable)
			.map_err(|_| Error::<T>::TransferFailed)?;

		Ok(())
	}

	fn settle_payment(sender: &T::AccountId, beneficiary: &T::AccountId, payment_id: &T::PaymentId) -> DispatchResult {
		Payment::<T>::try_mutate((sender, beneficiary, payment_id), |maybe_payment| -> DispatchResult {
			let payment = maybe_payment.as_mut().map_err(|_| Error::<T>::InvalidPayment)?;

			let reason = &HoldReason::TransferPayment.into();

			// Release sender fees recipients
			let (fee_sender_recipients, total_sender_fee_amount) =
				payment.fees_details.get_fees_details_for_sender()?;
			let total_sender_release = total_sender_fee_amount.saturating_add(payment.incentive_amount);

			T::Assets::release(payment.asset.clone(), reason, sender, total_sender_release, Exact)
				.map_err(|_| Error::<T>::ReleaseFailed)?;

			for (sender_fee_recipient_account, fee_amount) in fee_sender_recipients.iter() {
				T::Assets::transfer(
					payment.asset.clone(),
					sender,
					sender_fee_recipient_account,
					*fee_amount,
					Preserve,
				)
				.map_err(|_| Error::<T>::TransferFailed)?;
			}

			// Release the whole payment
			T::Assets::release(payment.asset.clone(), reason, beneficiary, payment.amount, Exact)
				.map_err(|_| Error::<T>::ReleaseFailed)?;

			let (fee_beneficiary_recipients, _) = payment.fees_details.get_fees_details_for_beneficiary()?;

			for (beneficiary_recipient_account, fee_amount) in fee_beneficiary_recipients.iter() {
				T::Assets::transfer(
					payment.asset.clone(),
					beneficiary,
					beneficiary_recipient_account,
					*fee_amount,
					Preserve,
				)
				.map_err(|_| Error::<T>::TransferFailed)?;
			}

			payment.state = PaymentState::Finished;
			*maybe_payment = Ok(payment.clone());

			Ok(())
		})?;
		Ok(())
	}

	fn schedule_enactment(
		index: ReferendumIndex,
		track: &TrackInfoOf<T, I>,
		desired: DispatchTime<BlockNumberFor<T>>,
		origin: PalletsOriginOf<T>,
		call: BoundedCallOf<T, I>,
	) {
		let now = frame_system::Pallet::<T>::block_number();
		let earliest_allowed = now.saturating_add(track.min_enactment_period);
		let desired = desired.evaluate(now);
		let ok = T::Scheduler::schedule_named(
			(ASSEMBLY_ID, "enactment", index).using_encoded(sp_io::hashing::blake2_256),
			DispatchTime::At(desired.max(earliest_allowed)),
			None,
			63,
			origin,
			call,
		)
		.is_ok();
		debug_assert!(ok, "LOGIC ERROR: bake_referendum/schedule_named failed");
	}
}
