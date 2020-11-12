create_enum_with_aux_fns!(
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
