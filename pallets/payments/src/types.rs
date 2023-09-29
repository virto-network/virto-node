#![allow(unused_qualifications)]
use crate::*;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, BoundedVec, Percent, Saturating};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

// This pallet's asset id and balance type.
pub type AssetIdOf<T> = <<T as Config>::Assets as FunsInspect<<T as frame_system::Config>::AccountId>>::AssetId;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type MaxFeesOf<T> = <T as Config>::MaxFees;
pub type BalanceOf<T> = <<T as Config>::Assets as FunsInspect<<T as frame_system::Config>::AccountId>>::Balance;
pub type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
pub type BoundedDataOf<T> = BoundedVec<u8, <T as Config>::MaxRemarkLength>;
pub type ChargableOnDisputes = bool;
pub type Fee<T> = (AccountIdOf<T>, BalanceOf<T>, ChargableOnDisputes);
pub type FeeDetails<T> = BoundedVec<Fee<T>, MaxFeesOf<T>>;
pub type CallOf<T> = <T as Config>::RuntimeCall;
pub type BoundedCallOf<T> = Bounded<CallOf<T>>;

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

/// Fee Handler trait that defines how to handle marketplace fees to every
/// payment/swap
pub trait FeeHandler<T: pallet::Config> {
	/// Get the distribution of fees to marketplace participants
	fn apply_fees(
		asset: &AssetIdOf<T>,
		sender: &T::AccountId,
		beneficiary: &T::AccountId,
		amount: &BalanceOf<T>,
		remark: Option<&[u8]>,
	) -> Fees<T>;
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug, TypeInfo, MaxEncodedLen)]
pub enum SubTypes<T: pallet::Config> {
	Fixed(T::AccountId, BalanceOf<T>, ChargableOnDisputes),
	Percentage(T::AccountId, Percent, ChargableOnDisputes),
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, MaxEncodedLen, TypeInfo, Debug)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound(T: pallet::Config))]
pub struct Fees<T: pallet::Config> {
	pub sender_pays: FeeDetails<T>,
	pub beneficiary_pays: FeeDetails<T>,
}

impl<T: pallet::Config> Fees<T> {
	pub fn get_fees_details(
		&self,
		is_sender: bool,
		is_dispute: bool,
	) -> Result<(Vec<Fee<T>>, BalanceOf<T>, BalanceOf<T>), DispatchError> {
		let fees = if is_sender {
			&self.sender_pays
		} else {
			&self.beneficiary_pays
		};
		Self::get_fees_details_per_role(fees, is_dispute)
	}

	pub fn get_fees_details_per_role(
		fees: &FeeDetails<T>,
		is_dispute: bool,
	) -> Result<(Vec<Fee<T>>, BalanceOf<T>, BalanceOf<T>), DispatchError> {
		let mut fees_per_account: BTreeMap<AccountIdOf<T>, Fee<T>> = BTreeMap::new();
		let mut total_to_discount: BalanceOf<T> = Zero::zero();
		let mut total_to_return: BalanceOf<T> = Zero::zero();

		for (account, fee, charged_dispute) in fees.iter() {
			if is_dispute {
				if *charged_dispute {
					total_to_discount = total_to_discount.saturating_add(*fee);
				} else {
					total_to_return = total_to_return.saturating_add(*fee);
				}
			} else {
				total_to_discount = total_to_discount.saturating_add(*fee);
			}

			let current_fee = fees_per_account
				.entry(account.clone())
				.or_insert_with(|| (account.clone(), Zero::zero(), *charged_dispute));
			let (_, current_balance, _) = current_fee;
			*current_balance = current_balance.saturating_add(*fee);
		}

		Ok((
			fees_per_account.into_iter().map(|(_, v)| v).collect(),
			total_to_discount,
			total_to_return,
		))
	}
}

/// Types of Tasks that can be scheduled in the pallet
#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug, TypeInfo, MaxEncodedLen)]
pub enum Task {
	// payment `from` to `to` has to be cancelled
	Cancel,
	Dispute,
}

/// Types of Tasks that can be scheduled in the pallet
#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug, TypeInfo, MaxEncodedLen)]
pub enum Role {
	// payment `from` to `to` has to be cancelled
	Sender,
	Beneficiary,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug, TypeInfo, MaxEncodedLen)]
pub struct DisputeResult {
	pub percent_beneficiary: Percent,
	pub in_favor_of: Role,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug, TypeInfo, MaxEncodedLen)]
pub struct DisputeResultWithResolver<DisputeResult, AccountId> {
	pub dispute_result: DisputeResult,
	pub dispute_resolver: AccountId,
}
