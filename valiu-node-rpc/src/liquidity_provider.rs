use crate::{ProviderMembers, ProviderMembersEventsDecoder, Tokens, TokensEventsDecoder};
use core::marker::PhantomData;
use parity_scale_codec::{Decode, Encode};
use substrate_subxt::system::{System, SystemEventsDecoder};
use valiu_node_commons::{Asset, OfferRate};

#[substrate_subxt::module]
pub trait LiquidityProvider: ProviderMembers + System + Tokens {
    type Asset: Decode + Default + Encode + Eq + PartialEq + Send + Sync + 'static;
    type Collateral: Decode + Default + Encode + Eq + PartialEq + Send + Sync + 'static;
}

#[derive(Clone, Debug, PartialEq, parity_scale_codec::Encode, substrate_subxt::Call)]
pub struct AttestCall<T: LiquidityProvider> {
    pub asset: Asset,
    pub balance: <T as Tokens>::Balance,
    pub offer_rates: Vec<OfferRate<<T as Tokens>::Balance>>,
    pub phantom: PhantomData<T>,
}

#[derive(Clone, Debug, Eq, PartialEq, parity_scale_codec::Decode, substrate_subxt::Event)]
pub struct AttestationEvent<T: LiquidityProvider> {
    pub account: <T as System>::AccountId,
    pub asset: Asset,
}

#[derive(Clone, Debug, PartialEq, parity_scale_codec::Encode, substrate_subxt::Call)]
pub struct MembersCall<T: LiquidityProvider> {
    pub phantom: PhantomData<T>,
}

#[derive(Clone, Debug, Eq, PartialEq, parity_scale_codec::Decode, substrate_subxt::Event)]
pub struct MembersEvent<T: LiquidityProvider> {
    pub members: Vec<<T as System>::AccountId>,
}

#[derive(Clone, Debug, Eq, PartialEq, parity_scale_codec::Decode, substrate_subxt::Event)]
pub struct TransferEvent<T: LiquidityProvider> {
    pub from: <T as System>::AccountId,
    pub to: <T as System>::AccountId,
    pub balance: <T as Tokens>::Balance,
}

#[derive(Clone, Debug, PartialEq, parity_scale_codec::Encode, substrate_subxt::Call)]
pub struct TransferCall<T: LiquidityProvider> {
    pub to: <T as System>::AccountId,
    pub to_amount: <T as Tokens>::Balance,
}
