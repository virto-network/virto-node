use core::fmt;

/// A resource or valuable thing.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
)]
pub enum Asset {
    Collateral(Collateral),
    Usdv,
}

impl Asset {
    /// String representation
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match *self {
            Self::Collateral(c) => c.as_str(),
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
    fn from(from: Collateral) -> Self {
        Asset::Collateral(from)
    }
}

create_enum_with_aux_fns!(
    /// An asset that backs or can be used as a security for other assets
    #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
    #[derive(
        Clone,
        Copy,
        Debug,
        Eq,
        Ord,
        PartialEq,
        PartialOrd,
        parity_scale_codec::Decode,
        parity_scale_codec::Encode,
    )]
    pub enum Collateral {
        Usdc -> "USDC",
    }
);

impl Default for Collateral {
    #[inline]
    fn default() -> Self {
        Self::Usdc
    }
}
