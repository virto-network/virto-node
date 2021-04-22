use parity_scale_codec::{Decode, Encode};

// The payment method that is taken for a cashin or cashout
// TODO: Replace with actual provider names
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PaymentMethod {
    BankX,
    BankY,
}

// A trait for querying rates supplied by an LP
pub trait RateProvider<T, M, Z, U> {
    fn get_rates(from: T, to: T, method: M, who: Z) -> Option<U>;
}