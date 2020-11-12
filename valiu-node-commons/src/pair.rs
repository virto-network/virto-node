use crate::Asset;
use arrayvec::ArrayString;

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
    #[inline]
    pub const fn new(base: Asset, quote: Asset) -> Self {
        Self { base, quote }
    }

    #[inline]
    pub const fn base(&self) -> Asset {
        self.base
    }

    #[inline]
    pub const fn quote(&self) -> Asset {
        self.quote
    }

    #[inline]
    pub fn to_string(&self) -> ArrayString<[u8; 11]> {
        let mut s = ArrayString::new();
        s.push_str(self.base.as_str());
        s.push('-');
        s.push_str(self.quote.as_str());
        s
    }
}

impl From<[Asset; 2]> for Pair {
    #[inline]
    fn from(from: [Asset; 2]) -> Self {
        Self::new(from[0], from[1])
    }
}
