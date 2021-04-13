use crate::ParachainInfo;
pub use codec::{Decode, Encode};
pub use cumulus_primitives_core::ParaId;
pub use frame_support::traits::Get;
pub use sp_runtime::traits::Convert;
pub use vln_primitives::{Asset, NetworkAsset};
pub use xcm::v0::{
    Junction::{GeneralKey, Parachain, Parent},
    MultiAsset,
    MultiLocation::{self, X1, X3},
};

pub struct CurrencyIdConvert;
impl Convert<NetworkAsset, Option<MultiLocation>> for CurrencyIdConvert {
    fn convert(id: NetworkAsset) -> Option<MultiLocation> {
        match id {
            NetworkAsset::DOT => Some(X1(Parent)),
            NetworkAsset::ACA | NetworkAsset::AUSD => Some(native_currency_location(id)),
        }
    }
}

impl Convert<MultiLocation, Option<NetworkAsset>> for CurrencyIdConvert {
    fn convert(location: MultiLocation) -> Option<NetworkAsset> {
        match location {
            X1(Parent) => Some(NetworkAsset::DOT),
            X3(Parent, Parachain { id }, GeneralKey(key))
                if ParaId::from(id) == ParachainInfo::get() =>
            {
                // decode the general key
                if let Ok(currency_id) = NetworkAsset::decode(&mut &key[..]) {
                    // check if `currency_id` is cross-chain asset
                    match currency_id {
                        NetworkAsset::ACA | NetworkAsset::AUSD => Some(currency_id),
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

impl Convert<MultiAsset, Option<NetworkAsset>> for CurrencyIdConvert {
    fn convert(asset: MultiAsset) -> Option<NetworkAsset> {
        if let MultiAsset::ConcreteFungible { id, amount: _ } = asset {
            Self::convert(id)
        } else {
            None
        }
    }
}

fn native_currency_location(id: NetworkAsset) -> MultiLocation {
    X3(
        Parent,
        Parachain {
            id: ParachainInfo::get().into(),
        },
        GeneralKey(id.encode()),
    )
}
