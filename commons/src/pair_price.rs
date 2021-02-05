use crate::Pair;

/// The buy and sell prices and an asset.
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
pub struct PairPrice<N> {
    buy: N,
    pair: Pair,
    sell: N,
}

impl<N> PairPrice<N> {
    /// Creates a new instance from a given `pair`, `buy` and `sell`.
    #[inline]
    pub fn new<I>(pair: I, buy: N, sell: N) -> Self
    where
        I: Into<Pair>,
    {
        Self {
            buy,
            pair: pair.into(),
            sell,
        }
    }

    /// Buy
    #[inline]
    pub const fn buy(&self) -> &N {
        &self.buy
    }

    /// Pair
    #[inline]
    pub const fn pair(&self) -> &Pair {
        &self.pair
    }

    /// Sell
    #[inline]
    pub const fn sell(&self) -> &N {
        &self.sell
    }
}
