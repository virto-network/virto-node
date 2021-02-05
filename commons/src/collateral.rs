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
        Usd -> "USD",
        Usdc -> "USDC",
    }
);

impl Default for Collateral {
    #[inline]
    fn default() -> Self {
        Self::Usd
    }
}
