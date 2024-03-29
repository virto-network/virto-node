use serde::{Deserialize, Serialize};

use core::ops::Add;
use cumulus_primitives_core::MultiLocation;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::traits::{MaybeEquivalence, Zero};
use xcm::v3::{
	Junction::{GeneralIndex, GlobalConsensus, PalletInstance, Parachain},
	Junctions, NetworkId,
};

#[derive(Copy, Clone, Debug, Encode, Decode, Serialize, Deserialize, MaxEncodedLen, Eq, PartialEq, TypeInfo)]
pub enum MultiLocationFungibleAsset {
	Me(u128),
	Ancestor {
		network: NetworkId,
		id: u32,
		pallet: u8,
		index: u128,
	},
	Sibling {
		id: u32,
		pallet: u8,
		index: u128,
	},
}

impl Default for MultiLocationFungibleAsset {
	fn default() -> Self {
		Self::zero()
	}
}

impl Zero for MultiLocationFungibleAsset {
	fn zero() -> Self {
		Self::Me(0)
	}

	fn is_zero(&self) -> bool {
		self.eq(&Self::zero())
	}
}

impl Add for MultiLocationFungibleAsset {
	type Output = MultiLocationFungibleAsset;

	/// [MultiLocationFungibleAsset] cannot be added
	/// since it's an immutable value. Instead, you'll get
	/// [MultiLocationFungibleAsset::zero()]
	fn add(self, _rhs: Self) -> Self::Output {
		Self::zero()
	}
}

impl From<u32> for MultiLocationFungibleAsset {
	fn from(value: u32) -> Self {
		MultiLocationFungibleAsset::Me(value.into())
	}
}

pub struct AsMultiLocationFungibleAsset;
impl MaybeEquivalence<MultiLocation, MultiLocationFungibleAsset> for AsMultiLocationFungibleAsset {
	fn convert(value: &MultiLocation) -> Option<MultiLocationFungibleAsset> {
		match *value {
			MultiLocation {
				parents: 2,
				interior:
					Junctions::X4(GlobalConsensus(network), Parachain(id), PalletInstance(pallet), GeneralIndex(index)),
			} => Some(MultiLocationFungibleAsset::Ancestor {
				network,
				id,
				pallet,
				index,
			}),
			MultiLocation {
				parents: 1,
				interior: Junctions::X3(Parachain(id), PalletInstance(pallet), GeneralIndex(index)),
			} => Some(MultiLocationFungibleAsset::Sibling { id, pallet, index }),
			MultiLocation {
				parents: 0,
				interior: Junctions::X2(PalletInstance(13), GeneralIndex(index)),
			} => Some(MultiLocationFungibleAsset::Me(index)),
			_ => None,
		}
	}

	fn convert_back(value: &MultiLocationFungibleAsset) -> Option<MultiLocation> {
		match *value {
			MultiLocationFungibleAsset::Me(index) => Some(MultiLocation {
				parents: 0,
				interior: Junctions::X2(PalletInstance(13), GeneralIndex(index)),
			}),
			MultiLocationFungibleAsset::Sibling { id, pallet, index } => Some(MultiLocation {
				parents: 1,
				interior: Junctions::X3(Parachain(id), PalletInstance(pallet), GeneralIndex(index)),
			}),
			MultiLocationFungibleAsset::Ancestor {
				network,
				id,
				pallet,
				index,
			} => Some(MultiLocation {
				parents: 2,
				interior: Junctions::X4(
					GlobalConsensus(network),
					Parachain(id),
					PalletInstance(pallet),
					GeneralIndex(index),
				),
			}),
		}
	}
}
