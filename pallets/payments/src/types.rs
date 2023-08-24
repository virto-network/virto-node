#![allow(unused_qualifications)]
use crate::{pallet, AccountIdOf, AssetIdOf, BalanceOf, MaxFeesOf};
use codec::{Decode, Encode, HasCompact, MaxEncodedLen};
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::TypeInfo;
use sp_runtime::{BoundedVec, Percent};

/// The PaymentDetail struct stores information about the payment/escrow
/// A "payment" in virto network is similar to an escrow, it is used to
/// guarantee proof of funds and can be released once an agreed upon condition
/// has reached between the payment creator and recipient. The payment lifecycle
/// is tracked using the state field.
#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound(T: pallet::Config))]
pub struct PaymentDetail<T: pallet::Config> {
	/// type of asset used for payment
	pub asset: AssetIdOf<T>,
	/// amount of asset used for payment
	pub amount: BalanceOf<T>,
	/// incentive amount that is credited to creator for resolving
	pub incentive_amount: BalanceOf<T>,
	/// enum to track payment lifecycle [Created, NeedsReview, RefundRequested,
	/// Requested]
	pub state: PaymentState<BlockNumberFor<T>>,
	/// account that can settle any disputes created in the payment
	pub resolver_account: T::AccountId,
	/// fee charged and recipient account details
	pub fees_details: Fees<T>,
}

/// The `PaymentState` enum tracks the possible states that a payment can be in.
/// When a payment is 'completed' or 'cancelled' it is removed from storage and
/// hence not tracked by a state.
#[derive(Clone, Encode, Decode, Eq, PartialEq, MaxEncodedLen, TypeInfo, Debug)]
pub enum PaymentState<BlockNumber> {
	/// Amounts have been reserved and waiting for release/cancel
	Created,
	/// A judge needs to review and release manually
	NeedsReview,
	/// The user has requested refund and will be processed by `BlockNumber`
	RefundRequested {
		cancel_block: BlockNumber,
	},
	/// The recipient of this transaction has created a request
	PaymentRequested,
	Finished,
}

/// DisputeResolver trait defines how to create/assign judges for solving
/// payment disputes
pub trait DisputeResolver<Account> {
	/// Returns an `Account`
	fn get_resolver_account() -> Account;
}

/// Fee Handler trait that defines how to handle marketplace fees to every
/// payment/swap
pub trait FeeHandler<T: pallet::Config> {
	/// Get the distribution of fees to marketplace participants
	fn apply_fees(
		sender: &T::AccountId,
		beneficiary: &T::AccountId,
		amount: &BalanceOf<T>,
		remark: Option<&[u8]>,
	) -> Fees<T>;
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug, TypeInfo, MaxEncodedLen)]
pub enum SubTypes<T: pallet::Config> {
	Fixed(T::AccountId, BalanceOf<T>),
	Percentage(T::AccountId, Percent),
}

pub type Fee<T> = (AccountIdOf<T>, BalanceOf<T>);
pub type FeeDetails<T> = BoundedVec<Fee<T>, MaxFeesOf<T>>;

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, MaxEncodedLen, TypeInfo, Debug)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound(T: pallet::Config))]
pub struct Fees<T: pallet::Config> {
	pub sender_pays: FeeDetails<T>,
	pub beneficiary_pays: FeeDetails<T>,
}

/// Types of Tasks that can be scheduled in the pallet
#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug, TypeInfo, MaxEncodedLen)]
pub enum Task {
	// payment `from` to `to` has to be cancelled
	Cancel,
}

/// The details of a scheduled task
#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug, TypeInfo, MaxEncodedLen)]
pub struct ScheduledTask<Time: HasCompact> {
	/// the type of scheduled task
	pub task: Task,
	/// the 'time' at which the task should be executed
	pub when: Time,
}
