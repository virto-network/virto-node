use substrate_subxt::system::{System, SystemEventsDecoder};
use parity_scale_codec::{Decode, Encode};

#[substrate_subxt::module]
pub trait Tokens: System {
    type Amount: Decode + Default + Encode + Eq + PartialEq + Send + Sync + 'static;
    type Balance: Decode + Default + Encode + Eq + PartialEq + Send + Sync + 'static;
    type CurrencyId: Decode + Default + Encode + Eq + PartialEq + Send + Sync + 'static;
}

#[derive(Clone, Debug, PartialEq, parity_scale_codec::Encode, substrate_subxt::Store)]
pub struct TotalIssuanceStore<T: Tokens> {
    #[store(returns = T::Balance)]
    /// CurrencyId to retrieve the issuance.
    pub currency_id: T::CurrencyId,
}