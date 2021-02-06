use core::fmt;
use sp_core::H256;

// BankDetailsCid should hold a sha256 cid from ipfs
pub type BankDetailsCid = H256;

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
pub enum Destination<AccountId> {
    // type to denote destinations on vln blockchain
    Vln(AccountId),
    // type to denote off chain bank transfer transations
    Bank(BankDetailsCid),
}

impl<AccountId> Destination<AccountId> {
    /// String representation
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Vln(_inner) => "Vln",
            Self::Bank(_inner) => "Bank",
        }
    }
}

impl<AccountId: Default> Default for Destination<AccountId> {
    #[inline]
    fn default() -> Self {
        Self::Vln(Default::default())
    }
}

impl<AccountId> fmt::Display for Destination<AccountId> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
