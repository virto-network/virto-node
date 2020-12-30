use core::marker::PhantomData;
use std::boxed::Box;
use substrate_subxt::system::{System, SystemEventsDecoder};
use valiu_node_commons::{Asset, DistributionStrategy, OfferRate};
use valiu_node_runtime_types::Balance;

#[substrate_subxt::module]
pub trait LiquidityProvider: System {}

#[derive(Clone, Debug, PartialEq, parity_scale_codec::Encode, substrate_subxt::Call)]
pub struct AttestCall<T: LiquidityProvider> {
    pub asset: Asset,
    pub balance: Balance,
    pub offer_rates: Vec<OfferRate<Balance>>,
    phantom: PhantomData<T>,
}

#[derive(Clone, Debug, PartialEq, parity_scale_codec::Encode, substrate_subxt::Call)]
pub struct TransferCall<T: LiquidityProvider> {
    pub to: <T as System>::AccountId,
    pub to_amount: Balance,
    pub ds: DistributionStrategy,
}
