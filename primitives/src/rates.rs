use parity_scale_codec::{Decode, Encode};
use sp_runtime::{traits::Saturating, FixedU128, Percent};

// The payment method that is taken for a cashin or cashout
// TODO: Replace with actual provider names
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PaymentMethod {
    BankX,
    BankY,
}

// A trait for querying rates supplied by an LP
pub trait RateProvider<F, T, M, Z, R, P> {
    fn get_rates(from: F, to: T, method: M, who: Z) -> Option<(R, P)>;
}

// A trait for adding the premium and rate to get final price
pub trait RatePremiumCalc<R, P> {
    fn combine_rates(rate: R, premium: P) -> R;
}

#[derive(Debug)]
pub struct DefaultRatePremiumCalc;
impl RatePremiumCalc<FixedU128, Percent> for DefaultRatePremiumCalc {
    fn combine_rates(rate: FixedU128, premium: Percent) -> FixedU128 {
        rate.saturating_mul(FixedU128::from(premium))
            .saturating_add(rate)
    }
}
