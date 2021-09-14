use frame_support::{
    traits::{InstanceFilter},
    pallet_prelude::MaxEncodedLen,
    RuntimeDebug,
};

/// Proxy type enum lists the type of calls that are supported by the proxy pallet
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    MaxEncodedLen,
    codec::Decode,
    codec::Encode,
    RuntimeDebug,
)]
pub enum ProxyType {
    Any,
}
impl Default for ProxyType {
    fn default() -> Self {
        Self::Any
    }
}
impl<Call> InstanceFilter<Call> for ProxyType {
    fn filter(&self, _c: &Call) -> bool {
        match self {
            ProxyType::Any => true,
        }
    }
    fn is_superset(&self, o: &Self) -> bool {
        self == &ProxyType::Any || self == o
    }
}
