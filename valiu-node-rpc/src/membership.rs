use alloc::boxed::Box;
use substrate_subxt::system::{System, SystemEventsDecoder};

#[substrate_subxt::module]
pub trait Membership: System {}

#[derive(Clone, Debug, PartialEq, parity_scale_codec::Encode, substrate_subxt::Call)]
pub struct AddMemberCall<T: Membership> {
    pub origin: <T as System>::Address,
    pub who: <T as System>::Address,
}
