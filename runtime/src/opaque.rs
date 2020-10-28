use super::*;

pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

/// Opaque block type.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// Opaque block identifier type.
pub type BlockId = generic::BlockId<Block>;
/// Opaque block header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: Aura,
        pub grandpa: Grandpa,
    }
}
