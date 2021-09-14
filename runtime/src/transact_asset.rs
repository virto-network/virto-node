#![allow(clippy::all)]
use codec::FullCodec;
use sp_runtime::traits::{Convert, MaybeSerializeDeserialize};
use sp_std::{
    cmp::{Eq, PartialEq},
    fmt::Debug,
    marker::PhantomData,
    prelude::*,
    result,
};
use xcm::v0::{Error as XcmError, MultiAsset, MultiLocation, Result};
use xcm_executor::{
    traits::{Convert as MoreConvert, MatchesFungible, TransactAsset},
    Assets,
};

// Asset transaction errors.
enum Error {
    /// Failed to match fungible.
    FailedToMatchFungible,
    /// `MultiLocation` to `AccountId` Conversion failed.
    AccountIdConversionFailed,
    /// `CurrencyId` conversion failed.
    CurrencyIdConversionFailed,
}

impl From<Error> for XcmError {
    fn from(e: Error) -> Self {
        match e {
            Error::FailedToMatchFungible => {
                XcmError::FailedToTransactAsset("FailedToMatchFungible")
            }
            Error::AccountIdConversionFailed => {
                XcmError::FailedToTransactAsset("AccountIdConversionFailed")
            }
            Error::CurrencyIdConversionFailed => {
                XcmError::FailedToTransactAsset("CurrencyIdConversionFailed")
            }
        }
    }
}

/// The `TransactAsset` implementation, to handle `MultiAsset` deposit/withdraw.
/// Note that teleport related functions are unimplemented.
///
/// If the asset is known, deposit/withdraw will be handled by `MultiCurrency`,
/// else by `UnknownAsset` if unknown.
pub struct NetworkAssetAdapter<
    Transfer,
    UnknownAsset,
    Match,
    AccountId,
    AccountIdConvert,
    CurrencyId,
    CurrencyIdConvert,
>(
    PhantomData<(
        Transfer,
        UnknownAsset,
        Match,
        AccountId,
        AccountIdConvert,
        CurrencyId,
        CurrencyIdConvert,
    )>,
);

impl<
        Mutate: frame_support::traits::fungibles::Mutate<AccountId>
            + frame_support::traits::fungibles::Inspect<AccountId, AssetId = CurrencyId>,
        UnknownAsset: orml_xcm_support::UnknownAsset,
        Match: MatchesFungible<Mutate::Balance>,
        AccountId: Clone,
        AccountIdConvert: MoreConvert<MultiLocation, AccountId>,
        CurrencyId: FullCodec + Eq + PartialEq + Copy + MaybeSerializeDeserialize + Debug,
        CurrencyIdConvert: Convert<MultiAsset, Option<CurrencyId>>,
    > TransactAsset
    for NetworkAssetAdapter<
        Mutate,
        UnknownAsset,
        Match,
        AccountId,
        AccountIdConvert,
        CurrencyId,
        CurrencyIdConvert,
    >
{
    fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> Result {
        match (
            AccountIdConvert::convert_ref(location),
            CurrencyIdConvert::convert(asset.clone()),
            Match::matches_fungible(&asset),
        ) {
            // known asset
            (Ok(who), Some(currency_id), Some(amount)) => {
                Mutate::mint_into(currency_id, &who, amount)
                    .map_err(|e| XcmError::FailedToTransactAsset(e.into()))
            }
            // unknown asset
            _ => UnknownAsset::deposit(asset, location)
                .map_err(|e| XcmError::FailedToTransactAsset(e.into())),
        }
    }

    fn withdraw_asset(
        asset: &MultiAsset,
        location: &MultiLocation,
    ) -> result::Result<Assets, XcmError> {
        let who = AccountIdConvert::convert_ref(location)
            .map_err(|_| XcmError::from(Error::AccountIdConversionFailed))?;
        let currency_id = CurrencyIdConvert::convert(asset.clone())
            .ok_or_else(|| XcmError::from(Error::CurrencyIdConversionFailed))?;
        let amount: Mutate::Balance = Match::matches_fungible(&asset)
            .ok_or_else(|| XcmError::from(Error::FailedToMatchFungible))?;
        Mutate::burn_from(currency_id, &who, amount)
            .map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;

        Ok(asset.clone().into())
    }
}
