#![allow(clippy::upper_case_acronyms, clippy::unnecessary_cast)]
use core::fmt;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::{convert::TryFrom, prelude::*};
/// A resource or valuable thing.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Decode, Encode, TypeInfo)]
pub enum Asset {
    Collateral(Collateral),
    Fiat(Fiat),
    Network(NetworkAsset),
    Usdv,
}

impl Asset {
    /// String representation
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match *self {
            Self::Collateral(c) => c.as_str(),
            Self::Fiat(f) => f.as_str(),
            Self::Network(n) => n.as_str(),
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
        USDC = "USDC",
    }
}

enum_with_aux_fns! {
    /// A currency issued by a goverment
    pub enum Fiat {
        COP = "COP",
        VEZ = "VEZ",
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Decode, Encode, TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NetworkAsset {
    KSM = 0,
    AUSD = 1,
    ACA = 2,
}

impl NetworkAsset {
    pub const fn as_str(&self) -> &'static str {
        match self {
            NetworkAsset::ACA => "ACA",
            NetworkAsset::AUSD => "AUSD",
            NetworkAsset::KSM => "KSM",
        }
    }
}

impl TryFrom<Vec<u8>> for Asset {
    type Error = ();
    fn try_from(v: Vec<u8>) -> Result<Asset, ()> {
        match v.as_slice() {
            b"ACA" => Ok(Asset::Network(NetworkAsset::ACA)),
            b"AUSD" => Ok(Asset::Network(NetworkAsset::AUSD)),
            b"KSM" => Ok(Asset::Network(NetworkAsset::KSM)),
            _ => Err(()),
        }
    }
}

impl From<NetworkAsset> for u32 {
    fn from(v: NetworkAsset) -> u32 {
        match v {
            NetworkAsset::ACA => 2,
            NetworkAsset::AUSD => 1,
            NetworkAsset::KSM => 0,
        }
    }
}
