use crate::Asset;

/// A pair compares the value of one asset (base) to another asset (quote).
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(
    Clone,
    Debug,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
)]
pub struct Pair {
    base: Asset,
    quote: Asset,
}

impl Pair {
    /// Creates a new instance from a given `base` and `quote`.
    #[inline]
    pub const fn new(base: Asset, quote: Asset) -> Self {
        Self { base, quote }
    }

    /// Base
    #[inline]
    pub const fn base(&self) -> Asset {
        self.base
    }

    /// Quote
    #[inline]
    pub const fn quote(&self) -> Asset {
        self.quote
    }
}

impl From<[Asset; 2]> for Pair {
    #[inline]
    fn from(from: [Asset; 2]) -> Self {
        Self::new(from[0], from[1])
    }
}
