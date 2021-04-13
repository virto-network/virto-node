use crate::ParachainInfo;
pub use codec::{Decode, Encode};
pub use cumulus_primitives_core::ParaId;
pub use frame_support::traits::Get;
pub use sp_runtime::traits::Convert;
pub use vln_primitives::{ForeignCurrencyId, TokenSymbol};
pub use xcm::v0::{
    Junction::{GeneralKey, Parachain, Parent},
    MultiAsset,
    MultiLocation::{self, X1, X3},
};

pub struct CurrencyIdConvert;
impl Convert<ForeignCurrencyId, Option<MultiLocation>> for CurrencyIdConvert {
    fn convert(id: ForeignCurrencyId) -> Option<MultiLocation> {
        use ForeignCurrencyId::Token;
        use TokenSymbol::*;
        match id {
            Token(DOT) => Some(X1(Parent)),
            Token(ACA) | Token(AUSD) => Some(native_currency_location(id)),
            _ => None,
        }
    }
}

impl Convert<MultiLocation, Option<ForeignCurrencyId>> for CurrencyIdConvert {
    fn convert(location: MultiLocation) -> Option<ForeignCurrencyId> {
        use ForeignCurrencyId::Token;
        use TokenSymbol::*;
        match location {
            X1(Parent) => Some(Token(DOT)),
            X3(Parent, Parachain { id }, GeneralKey(key))
                if ParaId::from(id) == ParachainInfo::get() =>
            {
                // decode the general key
                if let Ok(currency_id) = ForeignCurrencyId::decode(&mut &key[..]) {
                    // check if `currency_id` is cross-chain asset
                    match currency_id {
                        Token(ACA) | Token(AUSD) => Some(currency_id),
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

impl Convert<MultiAsset, Option<ForeignCurrencyId>> for CurrencyIdConvert {
    fn convert(asset: MultiAsset) -> Option<ForeignCurrencyId> {
        if let MultiAsset::ConcreteFungible { id, amount: _ } = asset {
            Self::convert(id)
        } else {
            None
        }
    }
}

fn native_currency_location(id: ForeignCurrencyId) -> MultiLocation {
    X3(
        Parent,
        Parachain {
            id: ParachainInfo::get().into(),
        },
        GeneralKey(id.encode()),
    )
}
