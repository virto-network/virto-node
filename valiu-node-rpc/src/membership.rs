use std::boxed::Box;
use substrate_subxt::system::{System, SystemEventsDecoder};

#[substrate_subxt::module]
pub trait ProviderMembers: System {}

#[derive(Clone, Debug, PartialEq, parity_scale_codec::Encode, substrate_subxt::Call)]
pub struct AddMemberCall<T: ProviderMembers> {
    pub who: <T as System>::AccountId,
}
