use parity_scale_codec::{Decode, Encode};
use sp_runtime::{traits::Saturating, FixedU128, Permill};

// type to represent the premium charged by provider
pub type RatePremiumType = sp_runtime::Permill;

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rates<Base, Quote> {
    pub pair: AssetPair<Base, Quote>,
    pub medium: PaymentMethod,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssetPair<Base, Quote> {
    pub base: Base,
    pub quote: Quote,
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
pub trait RateProvider<X, M, Z, R, P> {
    fn get_rates(pair: X, medium: M, who: Z) -> Option<(R, P)>;
}

// A trait for adding the premium and rate to get final price
pub trait RatePremiumCalc<R, P> {
    fn combine_rates(rate: R, premium: P) -> R;
}

#[derive(Debug)]
pub struct DefaultRatePremiumCalc;
impl RatePremiumCalc<FixedU128, Permill> for DefaultRatePremiumCalc {
    fn combine_rates(rate: FixedU128, premium: Permill) -> FixedU128 {
        rate.saturating_mul(FixedU128::from(premium))
            .saturating_add(rate)
    }
}
