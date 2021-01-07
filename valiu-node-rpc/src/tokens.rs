use core::fmt::Debug;
use parity_scale_codec::{Decode, Encode};
use substrate_subxt::system::{System, SystemEventsDecoder};
use valiu_node_commons::Asset;

#[substrate_subxt::module]
pub trait Tokens: System {
    type Amount: Decode + Default + Encode + Eq + PartialEq + Send + Sync + 'static;
    type Balance: Clone + Debug + Decode + Default + Encode + Eq + PartialEq + Send + Sync + 'static;
    type CurrencyId: Decode + Default + Encode + Eq + PartialEq + Send + Sync + 'static;
}

#[derive(Clone, Debug, Eq, PartialEq, parity_scale_codec::Decode, substrate_subxt::Event)]
pub struct TransferredEvent<T: Tokens> {
    pub currency_id: Asset,
    pub from: <T as System>::AccountId,
    pub to: <T as System>::AccountId,
    pub amount: T::Balance,
}
