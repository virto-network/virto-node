use crate::{impl_opaque_keys, Aura, Grandpa};
use alloc::vec::Vec;

impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: Aura,
        pub grandpa: Grandpa,
    }
}
