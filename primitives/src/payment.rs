#![allow(unused_qualifications)]
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::DispatchResult;

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PaymentDetail<Asset, Amount> {
	pub asset: Asset,
	pub amount: Amount,
	pub incentive_amount: Amount,
	pub state: PaymentState,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PaymentState {
	Created,
	Released,
	Cancelled,
	/// A judge needs to review and release manually
	NeedsReview,
}

// trait that defines how to create/release payments for users
pub trait PaymentHandler<Account, Asset, Amount> {
	/// Attempt to reserve an amount of the given asset from the caller
	/// If not possible then return Error. Possible reasons for failure include:
	/// - User does not have enough balance.
	fn create_payment(from: Account, to: Account, asset: Asset, amount: Amount) -> DispatchResult;

	/// Attempt to transfer an amount of the given asset from the given payment_id
	/// If not possible then return Error. Possible reasons for failure include:
	/// - The payment does not exist
	/// - The unreserve operation fails
	/// - The transfer operation fails
	fn release_payment(from: Account, to: Account) -> DispatchResult;

	/// Attempt to cancel a payment in Created state. This will set the payment
	/// state to cancel and release the reserved amount back to the creator.
	/// If not possible then return Error. Possible reasons for failure include:
	/// - The payment does not exist
	/// - The payment is not in Created state
	/// - The unreserve operation fails
	fn cancel_payment(from: Account, to: Account) -> DispatchResult;

	/// Attempt to fetch the details of a payment from the given payment_id
	/// Possible reasons for failure include:
	/// - The payment does not exist
	fn get_payment_details(from: Account, to: Account) -> Option<PaymentDetail<Asset, Amount>>;
}
