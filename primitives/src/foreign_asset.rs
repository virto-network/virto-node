// Represents assets from other parachains that are supported in vln parachain
// Ref : https://github.com/laminar-protocol/laminar-chain/blob/a07ea4aa75bce5d30a24ce2e7a506dda5e22013f/primitives/src/lib.rs#L101
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::{convert::TryFrom, prelude::*, vec};
use parity_scale_codec::{Decode, Encode, Error, Input};
use sp_runtime::{
 RuntimeDebug
};

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ForeignCurrencyId {
	LAMI = 0,
	AUSD,
	DOT,
}

impl TryFrom<Vec<u8>> for ForeignCurrencyId {
	type Error = ();
	fn try_from(v: Vec<u8>) -> Result<ForeignCurrencyId, ()> {
		match v.as_slice() {
			b"LAMI" => Ok(ForeignCurrencyId::LAMI),
			b"AUSD" => Ok(ForeignCurrencyId::AUSD),
			b"DOT" => Ok(ForeignCurrencyId::DOT),
			_ => Err(()),
		}
	}
}