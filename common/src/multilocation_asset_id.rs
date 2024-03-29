#[cfg(feature = "runtime")]
use {
	frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo},
	serde::{Deserialize, Serialize},
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
	feature = "runtime",
	derive(Encode, Decode, Serialize, Deserialize, MaxEncodedLen, TypeInfo)
)]
pub enum FungibleAssetLocation {
	Here(u32),
	Sibling {
		id: u16,
		pallet: u8,
		index: u32,
	},
	External {
		network: NetworkId,
		id: u16,
		pallet: u8,
		index: u32,
	},
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
	feature = "runtime",
	derive(Encode, Decode, Serialize, Deserialize, MaxEncodedLen, TypeInfo)
)]
pub enum NetworkId {
	Polkadot,
	Kusama,
	Ethereum { chain_id: u64 },
}

impl Default for FungibleAssetLocation {
	fn default() -> Self {
		Self::Here(0)
	}
}

impl From<u32> for FungibleAssetLocation {
	fn from(value: u32) -> Self {
		FungibleAssetLocation::Here(value)
	}
}

#[cfg(feature = "runtime")]
pub mod runtime {
	use super::FungibleAssetLocation;
	use core::ops::Add;
	use cumulus_primitives_core::MultiLocation;
	use sp_runtime::traits::{MaybeEquivalence, Zero};
	use xcm::v3::{
		Junction::{GeneralIndex, GlobalConsensus, PalletInstance, Parachain},
		Junctions, NetworkId,
	};

	impl Zero for FungibleAssetLocation {
		fn zero() -> Self {
			Self::Here(0)
		}

		fn is_zero(&self) -> bool {
			self.eq(&Self::zero())
		}
	}

	impl Add for FungibleAssetLocation {
		type Output = FungibleAssetLocation;

		/// [FungibleAssetLocation] cannot be added
		/// since it's an immutable value. Instead, you'll get
		/// [FungibleAssetLocation::zero()]
		fn add(self, _rhs: Self) -> Self::Output {
			Self::zero()
		}
	}

	impl TryFrom<NetworkId> for super::NetworkId {
		type Error = &'static str;

		fn try_from(value: NetworkId) -> Result<super::NetworkId, Self::Error> {
			match value {
				NetworkId::Polkadot => Ok(super::NetworkId::Polkadot),
				NetworkId::Kusama => Ok(super::NetworkId::Kusama),
				NetworkId::Ethereum { chain_id } => Ok(super::NetworkId::Ethereum { chain_id }),
				_ => Err("This network is not supported"),
			}
		}
	}

	impl From<super::NetworkId> for NetworkId {
		fn from(value: super::NetworkId) -> NetworkId {
			match value {
				super::NetworkId::Polkadot => NetworkId::Polkadot,
				super::NetworkId::Kusama => NetworkId::Kusama,
				super::NetworkId::Ethereum { chain_id } => NetworkId::Ethereum { chain_id },
			}
		}
	}

	pub struct AsFungibleAssetLocation;
	impl MaybeEquivalence<MultiLocation, FungibleAssetLocation> for AsFungibleAssetLocation {
		fn convert(value: &MultiLocation) -> Option<FungibleAssetLocation> {
			match *value {
				MultiLocation {
					parents: 2,
					interior:
						Junctions::X4(GlobalConsensus(network), Parachain(id), PalletInstance(pallet), GeneralIndex(index)),
				} => Some(FungibleAssetLocation::External {
					network: network.try_into().ok()?,
					id: id.try_into().ok()?,
					pallet,
					index: index.try_into().ok()?,
				}),
				MultiLocation {
					parents: 1,
					interior: Junctions::X3(Parachain(id), PalletInstance(pallet), GeneralIndex(index)),
				} => Some(FungibleAssetLocation::Sibling {
					id: id.try_into().ok()?,
					pallet,
					index: index.try_into().ok()?,
				}),
				MultiLocation {
					parents: 0,
					interior: Junctions::X2(PalletInstance(13), GeneralIndex(index)),
				} => Some(FungibleAssetLocation::Here(
					index.try_into().expect("as it is here, we the types will match; qed"),
				)),
				_ => None,
			}
		}

		fn convert_back(value: &FungibleAssetLocation) -> Option<MultiLocation> {
			match *value {
				FungibleAssetLocation::Here(index) => Some(MultiLocation {
					parents: 0,
					interior: Junctions::X2(PalletInstance(13), GeneralIndex(index.into())),
				}),
				FungibleAssetLocation::Sibling { id, pallet, index } => Some(MultiLocation {
					parents: 1,
					interior: Junctions::X3(Parachain(id.into()), PalletInstance(pallet), GeneralIndex(index.into())),
				}),
				FungibleAssetLocation::External {
					network,
					id,
					pallet,
					index,
				} => Some(MultiLocation {
					parents: 2,
					interior: Junctions::X4(
						GlobalConsensus(network.into()),
						Parachain(id.into()),
						PalletInstance(pallet),
						GeneralIndex(index.into()),
					),
				}),
			}
		}
	}
}
