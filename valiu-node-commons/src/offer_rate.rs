use crate::Asset;

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
pub struct OfferRate<B> {
    asset: Asset,
    rate: B,
}

impl<B> OfferRate<B> {
    #[inline]
    pub fn new(asset: Asset, rate: B) -> Self {
        Self { asset, rate }
    }

    #[inline]
    pub const fn asset(&self) -> Asset {
        self.asset
    }

    #[inline]
    pub const fn rate(&self) -> &B {
        &self.rate
    }
}
