use core::fmt;

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
pub enum ValiuCurrencies {
    Btc,
    Usdv,
}

impl ValiuCurrencies {
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match *self {
            Self::Btc => "BTC",
            Self::Usdv => "USDv",
        }
    }
}

impl fmt::Display for ValiuCurrencies {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
