use parity_scale_codec::{Decode, Encode};
use sp_runtime::{traits::Saturating, FixedU128, Permill};

// type to represent the premium charged by provider
pub type RatePremiumType = Permill;

#[allow(unused_qualifications)]
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RateDetail<T> {
    pub rate: T,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rates<B, Q> {
    pub pair: AssetPair<B, Q>,
    pub medium: PaymentMethod,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssetPair<B, Q> {
    pub base: B,
    pub quote: Q,
}

// The payment method that is taken for a cashin or cashout
// TODO: Replace with actual provider names
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PaymentMethod {
    BankX,
    BankY,
}

// A trait for querying rates supplied by an LP
pub trait RateProvider<X, M, Z, R> {
    fn get_rates(pair: X, medium: M, who: Z) -> Option<R>;
}

// A trait for adding the premium and rate to get final price
pub trait RateCombinator<R, P> {
    fn combine_rates(rate: R, premium: P) -> R;
}

#[derive(Debug)]
pub struct DefaultRateCombinator;
impl RateCombinator<FixedU128, Permill> for DefaultRateCombinator {
    fn combine_rates(rate: FixedU128, premium: Permill) -> FixedU128 {
        rate.saturating_mul(FixedU128::from(premium))
            .saturating_add(rate)
    }
}
