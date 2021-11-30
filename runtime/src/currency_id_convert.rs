use crate::ParachainInfo;
pub use codec::{Decode, Encode};
pub use cumulus_primitives_core::ParaId;
pub use frame_support::traits::Get;
pub use sp_runtime::traits::Convert;
pub use virto_primitives::{Asset, NetworkAsset};
pub use xcm::latest::prelude::*;

pub struct CurrencyIdConvert;
impl Convert<Asset, Option<MultiLocation>> for CurrencyIdConvert {
	fn convert(id: Asset) -> Option<MultiLocation> {
		match id {
			Asset::Network(NetworkAsset::KSM) => Some(MultiLocation::parent()),
			Asset::Network(NetworkAsset::KAR) | Asset::Network(NetworkAsset::KUSD) =>
				Some(native_currency_location(id)),
			_ => None,
		}
	}
}

impl Convert<MultiLocation, Option<Asset>> for CurrencyIdConvert {
	fn convert(location: MultiLocation) -> Option<Asset> {
		if location == MultiLocation::parent() {
			Some(Asset::Network(NetworkAsset::KSM))
		}
		// TODO : handle external parachain token support
		else {
			None
		}
	}
}

impl Convert<MultiAsset, Option<Asset>> for CurrencyIdConvert {
	fn convert(asset: MultiAsset) -> Option<Asset> {
		if let MultiAsset { id: Concrete(location), .. } = asset {
			Self::convert(location)
		} else {
			None
		}
	}
}

fn native_currency_location(id: Asset) -> MultiLocation {
	MultiLocation::new(1, X2(Parachain(ParachainInfo::get().into()), GeneralKey(id.encode())))
}
