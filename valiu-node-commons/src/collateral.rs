macro_rules! count_idents {
    ($($idents:ident)*) => {0usize $(+ replace_expr!($idents 1usize))*};
}

macro_rules! create_enum_with_aux_fns {
    (
        $(#[$mac:meta])*
        $v:vis enum $name:ident {
          $($variant:ident($variant_name:literal),)*
        }
    ) => {
        $(#[$mac])*
        $v enum $name {
          $($variant,)*
        }

        impl $name {
            #[inline]
            pub const fn as_str(&self) -> &'static str {
                match *self {
                    $(Self::$variant => $variant_name,)*
                }
            }

            #[inline]
            pub const fn len() -> usize {
                count_idents!($($variant)*)
            }

            #[inline]
            pub const fn variants() -> [Self; Self::len()] {
                [$(Self::$variant,)*]
            }
        }

        impl core::fmt::Display for Collateral {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }
    }
}

macro_rules! replace_expr {
    ($_i:ident $sub:expr) => {
        $sub
    };
}

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
        Usd("USD"),
        Usdc("USDC"),
    }
);
