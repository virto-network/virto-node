#![allow(unused_qualifications)]
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::DispatchError;

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EscrowDetail<Asset, Amount> {
    pub asset: Asset,
    pub amount: Amount,
    pub state: EscrowState,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EscrowState {
    Created,
    Released,
    Cancelled,
}

// trait that defines how to create/release escrows for users
pub trait EscrowHandler<Account, Asset, Amount> {
    /// Attempt to reserve an amount of the given asset from the caller
    /// If not possible then return Error. Possible reasons for failure include:
    /// - User does not have enough balance.
    fn create_escrow(
        from: Account,
        to: Account,
        asset: Asset,
        amount: Amount,
    ) -> Result<(), DispatchError>;

    /// Attempt to transfer an amount of the given asset from the given escrow_id
    /// If not possible then return Error. Possible reasons for failure include:
    /// - The escrow does not exist
    /// - The unreserve operation fails
    /// - The transfer operation fails
    fn release_escrow(from: Account, to: Account) -> Result<(), DispatchError>;

    /// Attempt to cancel an escrow in Created state. This will set the escrow
    /// state to cancel and release the reserved amount back to the creator.
    /// If not possible then return Error. Possible reasons for failure include:
    /// - The escrow does not exist
    /// - The escrow is not in Created state
    /// - The unreserve operation fails
    fn cancel_escrow(from: Account, to: Account) -> Result<(), DispatchError>;

    /// Attempt to fetch the details of an escrow from the given escrow_id
    /// Possible reasons for failure include:
    /// - The escrow does not exist
    fn get_escrow_details(from: Account, to: Account) -> Option<EscrowDetail<Asset, Amount>>;
}
