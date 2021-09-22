use crate::ParachainInfo;
pub use codec::{Decode, Encode};
pub use cumulus_primitives_core::ParaId;
pub use frame_support::traits::Get;
pub use sp_runtime::traits::Convert;
pub use virto_primitives::{Asset, NetworkAsset};
pub use xcm::v0::{
    Junction::{GeneralKey, Parachain, Parent},
    MultiAsset,
    MultiLocation::{self, X1, X3},
};

pub struct CurrencyIdConvert;
impl Convert<Asset, Option<MultiLocation>> for CurrencyIdConvert {
    fn convert(id: Asset) -> Option<MultiLocation> {
        match id {
            Asset::Network(NetworkAsset::KSM) => Some(X1(Parent)),
            Asset::Network(NetworkAsset::KAR) | Asset::Network(NetworkAsset::KUSD) => {
                Some(native_currency_location(id))
            }
            _ => None,
        }
    }
}

impl Convert<MultiLocation, Option<Asset>> for CurrencyIdConvert {
    fn convert(location: MultiLocation) -> Option<Asset> {
        match location {
            X1(Parent) => Some(Asset::Network(NetworkAsset::KSM)),
            X3(Parent, Parachain(id), GeneralKey(key))
                if ParaId::from(id) == ParachainInfo::get() =>
            {
                // decode the general key
                if let Ok(currency_id) = Asset::decode(&mut &key[..]) {
                    // check if `currency_id` is cross-chain asset
                    match currency_id {
                        Asset::Network(NetworkAsset::KAR) | Asset::Network(NetworkAsset::KUSD) => {
                            Some(currency_id)
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Convert<MultiAsset, Option<Asset>> for CurrencyIdConvert {
    fn convert(asset: MultiAsset) -> Option<Asset> {
        if let MultiAsset::ConcreteFungible { id, amount: _ } = asset {
            Self::convert(id)
        } else {
            None
        }
    }
}

fn native_currency_location(id: Asset) -> MultiLocation {
    X3(
        Parent,
        Parachain(ParachainInfo::get().into()),
        GeneralKey(id.encode()),
    )
}
