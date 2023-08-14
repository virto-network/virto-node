#![cfg_attr(not(feature = "std"), no_std)]

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
	ensure,
	traits::{
		fungibles::{
			self,
			hold::{Inspect as FunHoldInspect, Mutate as FunHoldMutate},
			Balanced as FunBalanced, Inspect as FunInspect, Mutate as FunMutate,
		},
		tokens::{fungibles::Inspect as FunsInspect, Fortitude::Polite, Precision::Exact, Preservation::Preserve},
	},
	BoundedVec,
};
use sp_runtime::Saturating;

pub mod weights;
pub use weights::*;

use sp_runtime::{traits::StaticLookup, DispatchResult, Percent};

pub mod types;
pub use types::*;

// Type alias for `frame_system`'s account id.
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
// This pallet's asset id and balance type.
type AssetIdOf<T> = <<T as Config>::Assets as FunsInspect<<T as frame_system::Config>::AccountId>>::AssetId;
type BalanceOf<T> = <<T as Config>::Assets as FunsInspect<<T as frame_system::Config>::AccountId>>::Balance;
type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
pub type BoundedDataOf<T> = BoundedVec<u8, <T as Config>::MaxRemarkLength>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*,
		traits::{
			tokens::{Fortitude::Polite, Precision::Exact},
			Currency, EnsureOrigin,
			ExistenceRequirement::KeepAlive,
			Imbalance, OnUnbalanced, ReservableCurrency, WithdrawReasons,
		},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_core::Get;

	use sp_runtime::{traits::CheckedAdd, Percent};
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Currency type that this works on.
		type Assets: FunInspect<Self::AccountId, Balance = Self::AssetsBalance>
			+ FunMutate<Self::AccountId>
			+ FunBalanced<Self::AccountId>
			+ FunHoldInspect<Self::AccountId>
			+ FunHoldMutate<Self::AccountId, Reason = Self::RuntimeHoldReasons>
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

		type DisputeResolver: DisputeResolver<Self::AccountId>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[pallet::constant]
		type IncentivePercentage: Get<Percent>;

		/// The overarching hold reason.
		type RuntimeHoldReasons: Parameter + Member + MaxEncodedLen + Ord + Copy;

		#[pallet::constant]
		type MaxRemarkLength: Get<u32>;
		//type WeightInfo: WeightInfo;
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
		PaymentCreated {
			from: T::AccountId,
			asset: AssetIdOf<T>,
			amount: BalanceOf<T>,
			remark: Option<BoundedDataOf<T>>,
		},
		/// Payment amount released to the recipient
		PaymentReleased { from: T::AccountId, to: T::AccountId },
		/// Payment has been cancelled by the creator
		PaymentCancelled { from: T::AccountId, to: T::AccountId },
		/// A payment that NeedsReview has been resolved by Judge
		PaymentResolved {
			from: T::AccountId,
			to: T::AccountId,
			recipient_share: Percent,
		},
		/// the payment creator has created a refund request
		PaymentCreatorRequestedRefund {
			from: T::AccountId,
			to: T::AccountId,
			expiry: BlockNumberFor<T>,
		},
		/// the refund request from creator was disputed by recipient
		PaymentRefundDisputed { from: T::AccountId, to: T::AccountId },
		/// Payment request was created by recipient
		PaymentRequestCreated { from: T::AccountId, to: T::AccountId },
		/// Payment request was completed by sender
		PaymentRequestCompleted { from: T::AccountId, to: T::AccountId },
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
			recipient: T::AccountId,
			asset: AssetIdOf<T>,
			#[pallet::compact] amount: BalanceOf<T>,
			remark: Option<BoundedDataOf<T>>,
			reason: T::RuntimeHoldReasons,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// create PaymentDetail and add to storage
			let payment_detail = Self::create_payment(
				&who,
				&recipient,
				asset.clone(),
				amount,
				PaymentState::Created,
				T::IncentivePercentage::get(),
				remark.as_ref().map(|x| x.as_slice()),
			)?;
			// reserve funds for payment
			Self::reserve_payment_amount(&who, &recipient, payment_detail, &reason)?;
			// emit paymentcreated event
			Self::deposit_event(Event::PaymentCreated {
				from: who,
				asset,
				amount,
				remark,
			});
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// The function will create a new payment. The fee and incentive
	/// amounts will be calculated and the `PaymentDetail` will be added to
	/// storage.
	fn create_payment(
		from: &T::AccountId,
		recipient: &T::AccountId,
		asset: AssetIdOf<T>,
		amount: BalanceOf<T>,
		payment_state: PaymentState<T>,
		incentive_percentage: Percent,
		remark: Option<&[u8]>,
	) -> Result<PaymentDetail<T>, sp_runtime::DispatchError> {
		Payment::<T>::try_mutate(
			from,
			recipient,
			|maybe_payment| -> Result<PaymentDetail<T>, sp_runtime::DispatchError> {
				// only payment requests can be overwritten
				if let Some(payment) = maybe_payment {
					ensure!(
						payment.state == PaymentState::PaymentRequested,
						Error::<T>::PaymentAlreadyInProcess
					);
				}

				// Calculate incentive amount - this is to insentivise the user to release
				// the funds once a transaction has been completed
				let incentive_amount = incentive_percentage.mul_floor(amount);
				println!("incentive_amount: {:?}", incentive_amount);
				let mut new_payment = PaymentDetail {
					asset,
					amount,
					incentive_amount,
					state: payment_state,
					resolver_account: T::DisputeResolver::get_resolver_account(),
					fee_detail: None,
				};

				// Calculate fee amount - this will be implemented based on the custom
				// implementation of the fee provider
				let (fee_recipient, fee_percent) = T::FeeHandler::apply_fees(from, recipient, &new_payment, remark);
				println!("fee_recipient: {:?}, fee_percent: {:?}", fee_recipient, fee_percent);
				let fee_amount = fee_percent.mul_floor(amount);
				new_payment.fee_detail = Some((fee_recipient, fee_amount));

				*maybe_payment = Some(new_payment.clone());

				Ok(new_payment)
			},
		)
	}

	fn reserve_payment_amount(
		from: &T::AccountId,
		to: &T::AccountId,
		payment: PaymentDetail<T>,
		reason: &T::RuntimeHoldReasons,
	) -> DispatchResult {
		let fee_amount = payment.fee_detail.map(|(_, f)| f).unwrap_or_else(|| 0u32.into());

		let total_fee_amount = payment.incentive_amount.saturating_add(fee_amount);
		let total_amount = total_fee_amount.saturating_add(payment.amount);

		T::Assets::transfer_and_hold(payment.asset, reason, from, to, total_amount, Exact, Preserve, Polite)?;

		Ok(())
	}
}
