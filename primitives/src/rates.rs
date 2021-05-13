#![allow(unused_qualifications)]
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{traits::Saturating, FixedPointNumber, FixedU128, Permill};

// type to represent the premium charged by provider
pub type RatePremiumType = Permill;

// type to represent the fixed rate
pub type RateFixedType = FixedU128;

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RateDetail<FixedType, PremiumType> {
    Fixed(FixedType),
    Premium(PremiumType),
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssetPair<B, Q> {
    pub base: B,
    pub quote: Q,
}

// The payment method that is taken for a cashin or cashout
// TODO: Replace with actual provider names
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PaymentMethod {
    BankX,
    BankY,
}

// A trait for querying rates supplied by an LP
pub trait RateProvider<P, M, W> {
    type Rate: FixedPointNumber;
    // fetch rate for a given combo of pair/medium/provider
    fn get_rates(pair: P, medium: M, who: W) -> Option<Self::Rate>;
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
