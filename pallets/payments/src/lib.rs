#![cfg_attr(not(feature = "std"), no_std)]

use frame_benchmarking::Zero;
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
	ensure,
	traits::{
		fungibles::{
			self,
			hold::{Inspect as FunHoldInspect, Mutate as FunHoldMutate},
			Balanced as FunBalanced, Inspect as FunInspect, Mutate as FunMutate,
		},
		tokens::{fungibles::Inspect as FunsInspect, Fortitude::Polite, Precision::Exact, Preservation::Preserve},
		Get,
	},
	BoundedVec,
};

use sp_runtime::{DispatchError, Saturating};

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
pub type BoundedFeeDetails<T> =
	BoundedVec<(Role, <T as frame_system::Config>::AccountId, BalanceOf<T>), <T as Config>::MaxFees>;
pub type PaymentDetailtOf<T> = PaymentDetail<
	AssetIdOf<T>,
	BalanceOf<T>,
	<T as frame_system::Config>::AccountId,
	BlockNumberFor<T>,
	BoundedFeeDetails<T>,
>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, PalletId};
	use frame_system::pallet_prelude::*;

	use sp_runtime::{
		traits::{CheckedAdd, Get},
		Percent,
	};
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

		type FeeHandler: FeeHandler<Self, BoundedFeeDetails<Self>>;

		type DisputeResolver: DisputeResolver<Self::AccountId>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[pallet::constant]
		type IncentivePercentage: Get<Percent>;

		/// The overarching hold reason.
		type RuntimeHoldReasons: Parameter + Member + MaxEncodedLen + Ord + Copy;

		#[pallet::constant]
		type MaxRemarkLength: Get<u32>;

		#[pallet::constant]
		type MaxFees: Get<u32>;

		#[pallet::constant]
		type MaxDiscounts: Get<u32>;
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
		PaymentDetail<AssetIdOf<T>, BalanceOf<T>, T::AccountId, BlockNumberFor<T>, BoundedFeeDetails<T>>,
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
		/// Release was not possible
		ReleaseFailed,
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
			println!("payment_detail: {:?}", payment_detail);
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

		/// Release any created payment, this will transfer the reserved amount
		/// from the creator of the payment to the assigned recipient
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn release(
			origin: OriginFor<T>,
			to: T::AccountId,
			reason: T::RuntimeHoldReasons,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;

			// ensure the payment is in Created state
			let payment = Payment::<T>::get(&from, &to).ok_or(Error::<T>::InvalidPayment)?;
			ensure!(payment.state == PaymentState::Created, Error::<T>::InvalidAction);

			Self::settle_payment(&from, &to, &reason)?;

			Self::deposit_event(Event::PaymentReleased { from, to });
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// The function will create a new payment. The fee and incentive
	/// amounts will be calculated and the `PaymentDetail` will be added to
	/// storage.
	fn create_payment(
		sender: &T::AccountId,
		beneficiary: &T::AccountId,
		asset: AssetIdOf<T>,
		amount: BalanceOf<T>,
		payment_state: PaymentState<BlockNumberFor<T>>,
		incentive_percentage: Percent,
		remark: Option<&[u8]>,
	) -> Result<PaymentDetailtOf<T>, sp_runtime::DispatchError> {
		Payment::<T>::try_mutate(
			sender,
			beneficiary,
			|maybe_payment| -> Result<PaymentDetailtOf<T>, sp_runtime::DispatchError> {
				if let Some(payment) = maybe_payment {
					ensure!(
						payment.state == PaymentState::PaymentRequested,
						Error::<T>::PaymentAlreadyInProcess
					);
				}

				let incentive_amount = incentive_percentage.mul_floor(amount);

				let fees_details: Fees<BoundedFeeDetails<T>> =
					T::FeeHandler::apply_fees(&sender, &beneficiary, &amount, remark);

				let new_payment = PaymentDetailtOf::<T> {
					asset,
					amount,
					incentive_amount,
					state: payment_state,
					resolver_account: T::DisputeResolver::get_resolver_account(),
					fees_details,
				};

				*maybe_payment = Some(new_payment.clone());

				Ok(new_payment)
			},
		)
	}

	fn reserve_payment_amount(
		sender: &T::AccountId,
		beneficiary: &T::AccountId,
		payment: PaymentDetailtOf<T>,
		reason: &T::RuntimeHoldReasons,
	) -> DispatchResult {
		let handle_fees = |payer: &T::AccountId,
		                   fees: &BoundedVec<(Role, T::AccountId, BalanceOf<T>), T::MaxFees>|
		 -> Result<BalanceOf<T>, DispatchError> {
			let mut total_fee_for_payer: BalanceOf<T> = Zero::zero();

			for (role, account, _amount) in fees.iter() {
				let account_fee = fees
					.iter()
					.filter(|(r, a, _)| r == role && a == account)
					.map(|(_, _, fee)| *fee)
					.fold(Zero::zero(), |acc: BalanceOf<T>, x| acc.saturating_add(x));

				println!(
					"account_fee: {:?}, account: {:?}, payer: {:?}",
					account_fee, account, payer
				);

				T::Assets::transfer_and_hold(
					payment.asset.clone(),
					reason,
					payer,
					account,
					account_fee,
					Exact,
					Preserve,
					Polite,
				)?;

				total_fee_for_payer = total_fee_for_payer.saturating_add(account_fee);
				println!("account: {:?}, total_fee_for_payer: {:?}", account, total_fee_for_payer);
			}

			Ok(total_fee_for_payer)
		};

		let sender_total_fee = handle_fees(sender, &payment.fees_details.sender_pays)?;
		println!("****sender_total_fee: {:?}", sender_total_fee);
		let beneficiary_total_fee = handle_fees(beneficiary, &payment.fees_details.beneficiary_pays)?;
		println!("****beneficiary_total_fee: {:?}", beneficiary_total_fee);

		let total_fee_amount = sender_total_fee.saturating_add(beneficiary_total_fee);
		let total_amount = payment
			.incentive_amount
			.saturating_add(total_fee_amount)
			.saturating_add(payment.amount);

		T::Assets::hold(payment.asset.clone(), reason, sender, payment.incentive_amount)?;
		T::Assets::transfer_and_hold(
			payment.asset,
			reason,
			sender,
			beneficiary,
			total_amount,
			Exact,
			Preserve,
			Polite,
		)?;

		Ok(())
	}

	fn settle_payment(
		source: &T::AccountId,
		beneficiary: &T::AccountId,
		reason: &T::RuntimeHoldReasons,
	) -> DispatchResult {
		Payment::<T>::try_mutate(source, beneficiary, |maybe_payment| -> DispatchResult {
			let payment = maybe_payment.take().ok_or(Error::<T>::InvalidPayment)?;

			let mut total_amount = payment.amount;

			T::Assets::release(payment.asset.clone(), reason, &beneficiary, payment.amount, Exact)
				.map_err(|_| Error::<T>::ReleaseFailed)?;

			// Process sender fees
			for (role, fee_recipient, fee_amount) in payment.fees_details.sender_pays.iter() {
				total_amount = total_amount.saturating_sub(*fee_amount);
				T::Assets::transfer(payment.asset.clone(), beneficiary, fee_recipient, *fee_amount, Preserve)?;
			}

			// Process beneficiary fees
			for (role, fee_recipient, fee_amount) in payment.fees_details.beneficiary_pays.iter() {
				total_amount = total_amount.saturating_sub(*fee_amount);
				T::Assets::transfer(payment.asset.clone(), beneficiary, fee_recipient, *fee_amount, Preserve)?;
			}

			// release the incentive amount from the source account
			T::Assets::release(payment.asset, reason, &source, payment.incentive_amount, Exact)
				.map_err(|_| Error::<T>::ReleaseFailed)?;

			Ok(())
		})?;
		Ok(())
	}

	/* 	fn get_payment_details(from: &T::AccountId, to: &T::AccountId) -> Option<PaymentDetail<T, FeeDetails<T>>> {
		Payment::<T>::get(from, to)
	} */
}
