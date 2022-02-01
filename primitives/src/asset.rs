#![allow(clippy::upper_case_acronyms, clippy::unnecessary_cast)]
use core::fmt;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_std::{convert::TryFrom, prelude::*};

/// A resource or valuable thing.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(
	Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Decode, Encode, MaxEncodedLen, TypeInfo,
)]
pub enum Asset {
	Network(NetworkAsset),
}

impl Asset {
	/// String representation
	#[inline]
	pub const fn as_str(&self) -> &'static str {
		match *self {
			Self::Network(n) => n.as_str(),
		}
	}
}

impl Default for Asset {
	#[inline]
	fn default() -> Self {
		Self::Network(NetworkAsset::KSM)
	}
}

impl fmt::Display for Asset {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

#[derive(
	Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Decode, Encode, MaxEncodedLen, TypeInfo,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NetworkAsset {
	KSM = 0,
	KUSD = 1,
	KAR = 2,
}

impl NetworkAsset {
	pub const fn as_str(&self) -> &'static str {
		match self {
			NetworkAsset::KAR => "KAR",
			NetworkAsset::KUSD => "KUSD",
			NetworkAsset::KSM => "KSM",
		}
	}
}

impl TryFrom<Vec<u8>> for Asset {
	type Error = ();
	fn try_from(v: Vec<u8>) -> Result<Asset, ()> {
		match v.as_slice() {
			b"KAR" => Ok(Asset::Network(NetworkAsset::KAR)),
			b"KUSD" => Ok(Asset::Network(NetworkAsset::KUSD)),
			b"KSM" => Ok(Asset::Network(NetworkAsset::KSM)),
			_ => Err(()),
		}
	}
}

impl From<NetworkAsset> for u32 {
	fn from(v: NetworkAsset) -> u32 {
		match v {
			NetworkAsset::KAR => 2,
			NetworkAsset::KUSD => 1,
			NetworkAsset::KSM => 0,
		}
	}
}

impl From<NetworkAsset> for Asset {
	fn from(a: NetworkAsset) -> Self {
		Asset::Network(a)
	}
}
