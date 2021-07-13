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
impl Convert<u32, Option<MultiLocation>> for CurrencyIdConvert {
    fn convert(id: u32) -> Option<MultiLocation> {
        match id {
            0 => Some(X1(Parent)),
            1 | 2 => Some(native_currency_location(id)),
            _ => None,
        }
    }
}

impl Convert<MultiLocation, Option<u32>> for CurrencyIdConvert {
    fn convert(location: MultiLocation) -> Option<u32> {
        match location {
            X1(Parent) => Some(0),
            X3(Parent, Parachain(id), GeneralKey(key))
                if ParaId::from(id) == ParachainInfo::get() =>
            {
                // decode the general key
                if let Ok(currency_id) = NetworkAsset::decode(&mut &key[..]) {
                    // check if `currency_id` is cross-chain asset
                    match currency_id {
                        NetworkAsset::KAR => Some(NetworkAsset::KAR.into()),
                        NetworkAsset::KUSD => Some(NetworkAsset::KUSD.into()),
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

impl Convert<MultiAsset, Option<u32>> for CurrencyIdConvert {
    fn convert(asset: MultiAsset) -> Option<u32> {
        if let MultiAsset::ConcreteFungible { id, amount: _ } = asset {
            Self::convert(id)
        } else {
            None
        }
    }
}

fn native_currency_location(id: u32) -> MultiLocation {
    X3(
        Parent,
        Parachain(ParachainInfo::get().into()),
        GeneralKey(id.encode()),
    )
}
