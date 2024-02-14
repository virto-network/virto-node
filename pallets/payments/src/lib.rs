#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::pallet_prelude::BlockNumberFor;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_io::hashing::blake2_256;

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
		Bounded, CallerTrait, QueryPreimage, StorePreimage,
	},
};
use sp_std::vec::Vec;

pub mod weights;
use sp_runtime::{
	traits::{CheckedAdd, Dispatchable, StaticLookup},
	DispatchError, DispatchResult, Percent, Saturating,
};
pub use weights::*;

pub mod types;
pub use types::*;

pub trait PaymentId<T: frame_system::Config>: Copy + Clone {
	fn next(sender: &T::AccountId, beneficiary: &T::AccountId) -> Option<Self>;
}

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::{StorageDoubleMap, *},
		PalletId,
	};
	use frame_system::pallet_prelude::*;

	use sp_runtime::{traits::Get, Percent};

	#[cfg(feature = "runtime-benchmarks")]
	pub trait BenchmarkHelper<AccountId, AssetId, Balance> {
		fn create_asset(id: AssetId, admin: AccountId, is_sufficient: bool, min_balance: Balance);
	}

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
			+ parity_scale_codec::FullCodec
			+ Copy
			+ MaybeSerializeDeserialize
			+ sp_std::fmt::Debug
			+ Default
			+ TypeInfo
			+ MaxEncodedLen;

		type FeeHandler: FeeHandler<Self>;

		type DisputeResolver: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

		type PaymentId: PaymentId<Self> + Member + Parameter + MaxEncodedLen;

		type Scheduler: ScheduleNamed<BlockNumberFor<Self>, CallOf<Self>, Self::PalletsOrigin, Hasher = Self::Hashing>;

		/// The preimage provider used to look up call hashes to get the call.
		type Preimages: QueryPreimage<H = Self::Hashing> + StorePreimage;

		/// The overarching hold reason.
		type RuntimeHoldReason: From<HoldReason>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[pallet::constant]
		type IncentivePercentage: Get<Percent>;

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

		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper: BenchmarkHelper<AccountIdOf<Self>, AssetIdOf<Self>, BalanceOf<Self>>;
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
	pub type Payment<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		// Sender
		T::AccountId,
		Twox64Concat,
		T::PaymentId,
		PaymentDetail<T>,
		ResultQuery<Error<T>::NonExistentStorageValue>,
	>;

	#[pallet::storage]
	pub type PaymentParties<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::PaymentId,
		// Sender, Beneficiary pair
		(T::AccountId, T::AccountId),
		ResultQuery<Error<T>::NonExistentStorageValue>,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new payment has been created
		PaymentCreated {
			payment_id: T::PaymentId,
			asset: AssetIdOf<T>,
			amount: BalanceOf<T>,
			remark: Option<BoundedDataOf<T>>,
		},
		/// Payment amount released to the recipient
		PaymentReleased { payment_id: T::PaymentId },
		/// Payment has been cancelled by the creator
		PaymentCancelled { payment_id: T::PaymentId },
		/// A payment that NeedsReview has been resolved by Judge
		PaymentResolved {
			payment_id: T::PaymentId,
			recipient_share: Percent,
		},
		/// the payment creator has created a refund request
		PaymentCreatorRequestedRefund {
			payment_id: T::PaymentId,
			expiry: BlockNumberFor<T>,
		},
		/// the payment was refunded
		PaymentRefunded { payment_id: T::PaymentId },
		/// the refund request from creator was disputed by recipient
		PaymentRefundDisputed { payment_id: T::PaymentId },
		/// Payment request was created by recipient
		PaymentRequestCreated { payment_id: T::PaymentId },
		/// Payment request was completed by sender
		PaymentRequestCompleted { payment_id: T::PaymentId },
		/// Payment disputed resolved
		PaymentDisputeResolved { payment_id: T::PaymentId },
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
		/// Storage Value does not exist
		NonExistentStorageValue,
		/// Unable to issue a payment id
		NoPaymentIdAvailable,
		/// Call from wrong beneficiary
		InvalidBeneficiary,
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
		#[pallet::weight(<T as Config>::WeightInfo::pay(remark.as_ref().map(|x| x.len() as u32).unwrap_or(0)))]
		pub fn pay(
			origin: OriginFor<T>,
			beneficiary: AccountIdLookupOf<T>,
			asset: AssetIdOf<T>,
			#[pallet::compact] amount: BalanceOf<T>,
			remark: Option<BoundedDataOf<T>>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let beneficiary = T::Lookup::lookup(beneficiary)?;

			// create PaymentDetail and add to storage
			let (payment_id, payment_detail) = Self::create_payment(
				&sender,
				beneficiary,
				asset.clone(),
				amount,
				PaymentState::Created,
				T::IncentivePercentage::get(),
				remark.as_ref().map(|x| x.as_slice()),
			)?;

			// reserve funds for payment
			Self::reserve_payment_amount(&sender, payment_detail)?;
			// emit paymentcreated event
			Self::deposit_event(Event::PaymentCreated {
				payment_id,
				asset,
				amount,
				remark,
			});
			Ok(().into())
		}

		/// Release any created payment, this will transfer the reserved amount
		/// from the creator of the payment to the assigned recipient
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::release())]
		pub fn release(origin: OriginFor<T>, payment_id: T::PaymentId) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			// ensure the payment is in Created state
			let payment = Payment::<T>::get(&sender, &payment_id).map_err(|_| Error::<T>::InvalidPayment)?;
			ensure!(payment.state == PaymentState::Created, Error::<T>::InvalidAction);
			Self::settle_payment(&sender, &payment.beneficiary, &payment_id, None)?;

			Self::deposit_event(Event::PaymentReleased { payment_id });
			Ok(().into())
		}

		/// Cancel a payment in created state, this will release the reserved
		/// back to creator of the payment. This extrinsic can only be called by
		/// the recipient of the payment
		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::cancel())]
		pub fn cancel(origin: OriginFor<T>, payment_id: T::PaymentId) -> DispatchResultWithPostInfo {
			let beneficiary = ensure_signed(origin)?;
			let (sender, b) = PaymentParties::<T>::get(&payment_id)?;
			ensure!(beneficiary == b, Error::<T>::InvalidBeneficiary);

			let payment = Payment::<T>::get(&sender, &payment_id).map_err(|_| Error::<T>::InvalidPayment)?;

			match payment.state {
				PaymentState::Created => {
					Self::cancel_payment(&sender, payment)?;
					Self::deposit_event(Event::PaymentCancelled { payment_id });
				}
				PaymentState::RefundRequested { cancel_block: _ } => {
					Self::cancel_payment(&sender, payment)?;
					Self::deposit_event(Event::PaymentRefunded { payment_id });
				}
				_ => fail!(Error::<T>::InvalidAction),
			}

			Payment::<T>::remove(&sender, &payment_id);
			PaymentParties::<T>::remove(payment_id);

			Ok(().into())
		}

		/// Allow the creator of a payment to initiate a refund that will return
		/// the funds after a configured amount of time that the receiver has to
		/// react and opose the request
		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::request_refund())]
		pub fn request_refund(origin: OriginFor<T>, payment_id: T::PaymentId) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin.clone())?;

			let expiry = Payment::<T>::try_mutate(&sender, &payment_id, |maybe_payment| -> Result<_, DispatchError> {
				// ensure the payment exists
				let payment = maybe_payment.as_mut().map_err(|_| Error::<T>::InvalidPayment)?;
				// refunds only possible for payments in created state
				ensure!(payment.state == PaymentState::Created, Error::<T>::InvalidAction);

				// set the payment to requested refund
				let current_block = frame_system::Pallet::<T>::block_number();
				let cancel_block = current_block
					.checked_add(&T::CancelBufferBlockLength::get())
					.ok_or(Error::<T>::MathError)?;
				let cancel_call = <T as Config>::RuntimeCall::from(pallet::Call::<T>::cancel { payment_id });

				T::Scheduler::schedule_named(
					("payment", payment_id).using_encoded(blake2_256),
					DispatchTime::At(cancel_block),
					None,
					63,
					frame_system::RawOrigin::Signed(payment.beneficiary.clone()).into(),
					T::Preimages::bound(cancel_call)?,
				)?;

				payment.state = PaymentState::RefundRequested { cancel_block };

				Ok(cancel_block)
			})?;

			Self::deposit_event(Event::PaymentCreatorRequestedRefund { payment_id, expiry });

			Ok(().into())
		}

		/// Allow payment beneficiary to dispute the refund request from the
		/// payment creator This does not cancel the request, instead sends the
		/// payment to a NeedsReview state The assigned resolver account can
		/// then change the state of the payment after review.
		#[pallet::call_index(4)]
		#[pallet::weight(<T as Config>::WeightInfo::dispute_refund())]
		pub fn dispute_refund(origin: OriginFor<T>, payment_id: T::PaymentId) -> DispatchResultWithPostInfo {
			use PaymentState::*;
			let beneficiary = ensure_signed(origin)?;
			let (sender, b) = PaymentParties::<T>::get(&payment_id)?;
			ensure!(beneficiary == b, Error::<T>::InvalidBeneficiary);

			Payment::<T>::try_mutate(&sender, &payment_id, |maybe_payment| -> Result<_, DispatchError> {
				// ensure the payment exists
				let payment = maybe_payment.as_mut().map_err(|_| Error::<T>::InvalidPayment)?;

				// ensure the payment is in Requested Refund state
				let RefundRequested { cancel_block } = payment.state else {
					fail!(Error::<T>::InvalidAction);
				};
				ensure!(
					cancel_block > frame_system::Pallet::<T>::block_number(),
					Error::<T>::InvalidAction
				);

				// Hold beneficiary incentive amount to balance the incentives at the time to
				// resolve the dispute
				let reason = &HoldReason::TransferPayment.into();
				T::Assets::hold(payment.asset.clone(), reason, &beneficiary, payment.incentive_amount)?;

				payment.state = PaymentState::NeedsReview;

				T::Scheduler::cancel_named(("payment", payment_id).using_encoded(blake2_256))
			})?;

			Self::deposit_event(Event::PaymentRefundDisputed { payment_id });
			Ok(().into())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::resolve_dispute())]
		pub fn resolve_dispute(
			origin: OriginFor<T>,
			payment_id: T::PaymentId,
			dispute_result: DisputeResult,
		) -> DispatchResultWithPostInfo {
			let dispute_resolver = T::DisputeResolver::ensure_origin(origin)?;
			let (sender, beneficiary) = PaymentParties::<T>::get(&payment_id)?;

			let payment = Payment::<T>::get(&sender, &payment_id).map_err(|_| Error::<T>::InvalidPayment)?;
			ensure!(payment.state == PaymentState::NeedsReview, Error::<T>::InvalidAction);

			let dispute = Some((dispute_result, dispute_resolver));
			Self::settle_payment(&sender, &beneficiary, &payment_id, dispute)?;

			Self::deposit_event(Event::PaymentDisputeResolved { payment_id });
			Ok(().into())
		}

		// Creates a new payment with the given details. This can be called by the
		// recipient of the payment to create a payment and then completed by the sender
		// using the `accept_and_pay` extrinsic.  The payment will be in
		// PaymentRequested State and can only be modified by the `accept_and_pay`
		// extrinsic.
		#[pallet::call_index(6)]
		#[pallet::weight(<T as Config>::WeightInfo::request_payment())]
		pub fn request_payment(
			origin: OriginFor<T>,
			sender: AccountIdLookupOf<T>,
			asset: AssetIdOf<T>,
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let beneficiary = ensure_signed(origin)?;
			let sender = T::Lookup::lookup(sender)?;
			// create PaymentDetail and add to storage
			let (payment_id, _) = Self::create_payment(
				&sender,
				beneficiary,
				asset,
				amount,
				PaymentState::PaymentRequested,
				T::IncentivePercentage::get(),
				None,
			)?;

			Self::deposit_event(Event::PaymentRequestCreated { payment_id });

			Ok(().into())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(<T as Config>::WeightInfo::accept_and_pay())]
		pub fn accept_and_pay(origin: OriginFor<T>, payment_id: T::PaymentId) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let (_, beneficiary) = PaymentParties::<T>::get(&payment_id)?;

			Payment::<T>::try_mutate(&sender, payment_id, |maybe_payment| -> Result<_, DispatchError> {
				let payment = maybe_payment.as_mut().map_err(|_| Error::<T>::InvalidPayment)?;
				const IS_DISPUTE: bool = false;

				// Release sender fees recipients
				let (fee_sender_recipients, _total_sender_fee_amount_mandatory, _total_sender_fee_amount_optional) =
					payment.fees.summary_for(Role::Sender, IS_DISPUTE)?;

				let (
					fee_beneficiary_recipients,
					_total_beneficiary_fee_amount_mandatory,
					_total_beneficiary_fee_amount_optional,
				) = payment.fees.summary_for(Role::Beneficiary, IS_DISPUTE)?;

				Self::try_transfer_fees(&sender, payment, fee_sender_recipients, IS_DISPUTE)?;

				T::Assets::transfer(payment.asset.clone(), &sender, &beneficiary, payment.amount, Expendable)
					.map_err(|_| Error::<T>::TransferFailed)?;

				Self::try_transfer_fees(&beneficiary, payment, fee_beneficiary_recipients, IS_DISPUTE)?;

				payment.state = PaymentState::Finished;
				Ok(())
			})?;

			Self::deposit_event(Event::PaymentRequestCreated { payment_id });
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
		beneficiary: T::AccountId,
		asset: AssetIdOf<T>,
		amount: BalanceOf<T>,
		payment_state: PaymentState<BlockNumberFor<T>>,
		incentive_percentage: Percent,
		remark: Option<&[u8]>,
	) -> Result<(T::PaymentId, PaymentDetail<T>), DispatchError> {
		let payment_id = T::PaymentId::next(sender, &beneficiary).ok_or(Error::<T>::NoPaymentIdAvailable)?;
		Payment::<T>::try_mutate(sender, payment_id, |maybe_payment| -> Result<_, DispatchError> {
			if let Ok(payment) = maybe_payment {
				ensure!(
					payment.state == PaymentState::PaymentRequested,
					Error::<T>::PaymentAlreadyInProcess
				);
			}

			let incentive_amount = incentive_percentage.mul_floor(amount);

			let fees_details: Fees<T> = T::FeeHandler::apply_fees(&asset, sender, &beneficiary, &amount, remark);

			let new_payment = PaymentDetail::<T> {
				asset,
				amount,
				beneficiary: beneficiary.clone(),
				incentive_amount,
				state: payment_state,
				fees: fees_details,
			};
			*maybe_payment = Ok(new_payment.clone());
			PaymentParties::<T>::insert(payment_id, (sender, beneficiary));

			Ok(new_payment)
		})
		.map(|payment| (payment_id, payment))
	}

	fn reserve_payment_amount(sender: &T::AccountId, payment: PaymentDetail<T>) -> DispatchResult {
		let (_fee_recipients, total_fee_from_sender_mandatory, total_fee_from_sender_optional) =
			payment.fees.summary_for(Role::Sender, false)?;

		let total_hold_amount = total_fee_from_sender_mandatory
			.saturating_add(payment.incentive_amount)
			.saturating_add(total_fee_from_sender_optional);
		let reason = &HoldReason::TransferPayment.into();
		T::Assets::hold(payment.asset.clone(), reason, sender, total_hold_amount)?;

		T::Assets::transfer_and_hold(
			payment.asset,
			reason,
			sender,
			&payment.beneficiary,
			payment.amount,
			Exact,
			Preserve,
			Polite,
		)?;

		Ok(())
	}

	fn cancel_payment(sender: &T::AccountId, payment: PaymentDetail<T>) -> DispatchResult {
		let (_fee_recipients, total_fee_from_sender_mandatory, total_fee_from_sender_optional) =
			payment.fees.summary_for(Role::Sender, false)?;

		let total_hold_amount = total_fee_from_sender_mandatory
			.saturating_add(payment.incentive_amount)
			.saturating_add(total_fee_from_sender_optional);
		let reason = &HoldReason::TransferPayment.into();

		T::Assets::release(payment.asset.clone(), reason, sender, total_hold_amount, Exact)
			.map_err(|_| Error::<T>::ReleaseFailed)?;

		let beneficiary = &payment.beneficiary;
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
		maybe_dispute: Option<(DisputeResult, T::AccountId)>,
	) -> DispatchResult {
		Payment::<T>::try_mutate(sender, payment_id, |maybe_payment| -> DispatchResult {
			let payment = maybe_payment.as_mut().map_err(|_| Error::<T>::InvalidPayment)?;

			let reason = &HoldReason::TransferPayment.into();
			let is_dispute = maybe_dispute.is_some();

			// Release sender fees recipients
			let (fee_sender_recipients, total_sender_fee_amount_mandatory, total_sender_fee_amount_optional) =
				payment.fees.summary_for(Role::Sender, is_dispute)?;

			let total_sender_release = total_sender_fee_amount_mandatory
				.saturating_add(payment.incentive_amount)
				.saturating_add(total_sender_fee_amount_optional);

			T::Assets::release(payment.asset.clone(), reason, sender, total_sender_release, Exact)
				.map_err(|_| Error::<T>::ReleaseFailed)?;

			let (
				fee_beneficiary_recipients,
				_total_beneficiary_fee_amount_mandatory,
				_total_beneficiary_fee_amount_optional,
			) = payment.fees.summary_for(Role::Beneficiary, is_dispute)?;

			let mut beneficiary_release_amount = payment.amount;

			if is_dispute {
				beneficiary_release_amount = beneficiary_release_amount.saturating_add(payment.incentive_amount);
			}

			T::Assets::release(
				payment.asset.clone(),
				reason,
				beneficiary,
				beneficiary_release_amount,
				Exact,
			)
			.map_err(|_| Error::<T>::ReleaseFailed)?;

			Self::try_transfer_fees(sender, payment, fee_sender_recipients, is_dispute)?;

			Self::try_transfer_fees(beneficiary, payment, fee_beneficiary_recipients, is_dispute)?;

			if let Some((dispute_result, resolver)) = maybe_dispute {
				match dispute_result.in_favor_of {
					Role::Sender => {
						let amount_to_sender = dispute_result.percent_beneficiary.mul_floor(payment.amount);

						// Beneficiary looses the dispute and has to transfer the incentive_amount to
						// the dispute_resolver.
						T::Assets::transfer(
							payment.asset.clone(),
							beneficiary,
							&resolver,
							payment.incentive_amount,
							Expendable,
						)
						.map_err(|_| Error::<T>::TransferFailed)?;

						T::Assets::transfer(payment.asset.clone(), beneficiary, sender, amount_to_sender, Expendable)
							.map_err(|_| Error::<T>::TransferFailed)?;
					}
					Role::Beneficiary => {
						let amount_to_beneficiary = dispute_result.percent_beneficiary.mul_floor(payment.amount);
						let amount_to_sender = payment.amount.saturating_sub(amount_to_beneficiary);

						T::Assets::transfer(
							payment.asset.clone(),
							sender,
							&resolver,
							payment.incentive_amount,
							Expendable,
						)
						.map_err(|_| Error::<T>::TransferFailed)?;

						T::Assets::transfer(payment.asset.clone(), beneficiary, sender, amount_to_sender, Expendable)
							.map_err(|_| Error::<T>::TransferFailed)?;
					}
				}
			}

			payment.state = PaymentState::Finished;
			Ok(())
		})
	}

	fn try_transfer_fees(
		account: &T::AccountId,
		payment: &PaymentDetail<T>,
		fee_recipients: Vec<Fee<T>>,
		is_dispute: bool,
	) -> Result<(), sp_runtime::DispatchError> {
		for (recipient_account, fee_amount, mandatory) in fee_recipients.iter() {
			if !is_dispute || *mandatory {
				T::Assets::transfer(payment.asset.clone(), account, recipient_account, *fee_amount, Preserve)
					.map_err(|_| Error::<T>::TransferFailed)?;
			}
		}
		Ok(())
	}
}
