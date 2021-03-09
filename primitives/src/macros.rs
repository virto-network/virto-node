// Expands to 0 + 1 + 2 + 3 + ... + n through `replace_expr` calling
macro_rules! count_idents {
    ($($idents:ident)*) => {0usize $(+ replace_expr!($idents 1usize))*};
}
// Erases `$_i` and returns `$sub`
macro_rules! replace_expr {
    ($_i:ident $sub:expr) => {
        $sub
    };
}

macro_rules! enum_with_aux_fns {
    (
        $(#[$meta:meta])*
        $v:vis enum $name:ident {
          $($variant:ident = $variant_name:literal,)*
        }
    ) => {
        $(#[$meta])*
        #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, codec::Decode, codec::Encode)]
        $v enum $name {
          $($variant,)*
        }

        impl $name {
            /// String representation
            #[inline]
            pub const fn as_str(&self) -> &'static str {
                match *self {
                    $(Self::$variant => $variant_name,)*
                }
            }

            /// The number of variants
            #[inline]
            pub const fn len() -> usize {
                count_idents!($($variant)*)
            }

            /// An array that contains all variants
            #[inline]
            pub const fn variants() -> [Self; Self::len()] {
                [$(Self::$variant,)*]
            }
        }

        impl core::fmt::Display for $name {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        impl Default for $name {
            #[inline]
            fn default() -> Self {
                Self::variants()[0]
            }
        }
    }
}
