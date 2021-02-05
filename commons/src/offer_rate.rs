use crate::Asset;

/// How much someone is willing to exchange for an asset.
#[derive(
    Clone,
    Debug,
    Default,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
)]
pub struct OfferRate<B> {
    asset: Asset,
    rate: B,
}

impl<B> OfferRate<B> {
    /// Creates a new instance from a given `asset` and `rate`.
    #[inline]
    pub fn new(asset: Asset, rate: B) -> Self {
        Self { asset, rate }
    }

    /// Asset
    #[inline]
    pub const fn asset(&self) -> Asset {
        self.asset
    }

    /// Rate
    #[inline]
    pub const fn rate(&self) -> &B {
        &self.rate
    }
}
