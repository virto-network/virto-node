// Represents assets from other parachains that are supported in vln parachain
// Ref : https://github.com/laminar-protocol/laminar-chain/blob/a07ea4aa75bce5d30a24ce2e7a506dda5e22013f/primitives/src/lib.rs#L101
// Ref : https://github.com/open-web3-stack/open-runtime-module-library/wiki/xtokens
#![allow(clippy::upper_case_acronyms, clippy::unnecessary_cast)]
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;
use sp_std::{convert::TryFrom, prelude::*};

/// Supported token symbols.
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TokenSymbol {
    /// Acala native token.
    ACA = 0,
    /// Acala stable coin.
    AUSD = 1,
    /// Polkadot native token.
    DOT = 2,
    /// Valiu's USDV
    USDV = 3,
}

impl TryFrom<u8> for TokenSymbol {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(TokenSymbol::ACA),
            1 => Ok(TokenSymbol::AUSD),
            2 => Ok(TokenSymbol::DOT),
            3 => Ok(TokenSymbol::USDV),
            _ => Err(()),
        }
    }
}

/// Currency identifier.
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ForeignCurrencyId {
    /// Native token.
    Token(TokenSymbol),
}

impl TryFrom<Vec<u8>> for ForeignCurrencyId {
    type Error = ();
    fn try_from(v: Vec<u8>) -> Result<ForeignCurrencyId, ()> {
        match v.as_slice() {
            b"ACA" => Ok(ForeignCurrencyId::Token(TokenSymbol::ACA)),
            b"AUSD" => Ok(ForeignCurrencyId::Token(TokenSymbol::AUSD)),
            b"DOT" => Ok(ForeignCurrencyId::Token(TokenSymbol::DOT)),
            b"USDV" => Ok(ForeignCurrencyId::Token(TokenSymbol::USDV)),
            _ => Err(()),
        }
    }
}

impl Default for ForeignCurrencyId {
    #[inline]
    fn default() -> Self {
        Self::Token(TokenSymbol::USDV)
    }
}
