use core::fmt;
use parity_scale_codec as codec;

/// A resource or valuable thing.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, codec::Decode, codec::Encode)]
pub enum Asset {
    Collateral(Collateral),
    Fiat(Fiat),
    Usdv,
}

impl Asset {
    /// String representation
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match *self {
            Self::Collateral(c) => c.as_str(),
            Self::Fiat(f) => f.as_str(),
            Self::Usdv => "USDv",
        }
    }
}

impl Default for Asset {
    #[inline]
    fn default() -> Self {
        Self::Usdv
    }
}

impl fmt::Display for Asset {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<Collateral> for Asset {
    #[inline]
    fn from(c: Collateral) -> Self {
        Asset::Collateral(c)
    }
}

impl From<Fiat> for Asset {
    #[inline]
    fn from(f: Fiat) -> Self {
        Asset::Fiat(f)
    }
}

enum_with_aux_fns! {
    /// Asset used to back other assets
    pub enum Collateral {
        Usdc = "USDC",
    }
}

enum_with_aux_fns! {
    /// A currency issued by a goverment
    pub enum Fiat {
        Cop = "COP",
        Vez = "VEZ",
    }
}
