#![allow(unused_qualifications)]
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::DispatchError;

pub type EscrowId = u32;

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EscrowDetail<Recipent, Asset, Amount> {
    pub recipent: Recipent,
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
    ) -> Result<EscrowId, DispatchError>;

    /// Attempt to transfer an amount of the given asset from the given escrow_id
    /// If not possible then return Error. Possible reasons for failure include:
    /// - The escrow_id is not valid
    /// - The unreserve operation fails
    /// - The transfer operation fails
    fn release_escrow(from: Account, escrow_id: EscrowId) -> Result<(), DispatchError>;

    /// Attempt to fetch the details of an escrow from the given escrow_id
    /// Possible reasons for failure include:
    /// - The escrow_id is not valid
    fn get_escrow_details(
        from: Account,
        escrow_id: EscrowId,
    ) -> Option<EscrowDetail<Account, Asset, Amount>>;
}
