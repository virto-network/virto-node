use core::fmt;
use crate::runtime::AccountId;

pub type BtcTransactionId = [u8; 32];
pub type BankTransactionId = [u8; 32];

/// Identifier for different destinations on VLN
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
pub enum Destination {
    // type to denote destinations on vln blockchain
    Vln(AccountId),
    // type to denote transactions on the bitcoin blockchain
    Btc(BtcTransactionId),
    // type to denote off chain bank transfer transations
    Bank(BankTransactionId),
}

impl Destination {
    /// String representation
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match *self {
            Self::Vln(_inner) => "Vln",
            Self::Btc(_inner) => "Btc",
            Self::Bank(_inner) => "Bank"
        }
    }
}

impl Default for Destination {
    #[inline]
    fn default() -> Self {
        Self::Vln(Default::default())
    }
}

impl fmt::Display for Destination {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

