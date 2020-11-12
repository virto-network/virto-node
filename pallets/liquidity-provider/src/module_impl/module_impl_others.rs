use crate::{
    AccountRate, AccountRates, Balance, Module, OfferRateTy, ProviderMembers, RawEvent, Trait,
};
use alloc::vec::Vec;
use frame_support::{dispatch::DispatchResult, StorageDoubleMap};
use orml_traits::MultiCurrency;
use valiu_node_commons::Asset;

impl<T> Module<T>
where
    T: Trait,
{
    #[inline]
    pub(crate) fn do_attest(
        from: <T as frame_system::Trait>::AccountId,
        asset: Asset,
        balance: Balance<T>,
    ) -> DispatchResult {
        pallet_membership::Module::<T, ProviderMembers>::members()
            .binary_search(&from)
            .ok()
            .ok_or(pallet_membership::Error::<T, ProviderMembers>::NotMember)?;
        T::Asset::deposit(asset, &from, balance)?;
        Module::<T>::deposit_event(RawEvent::Attestation(from, asset));
        Ok(())
    }

    #[inline]
    pub(crate) fn update_account_rates(
        from: &<T as frame_system::Trait>::AccountId,
        asset: Asset,
        offer_rates: Vec<OfferRateTy<T>>,
    ) {
        for offer_rate in offer_rates {
            <AccountRates<T>>::mutate(asset, offer_rate.asset(), |account_rates| {
                let idx = account_rates
                    .binary_search_by(|el| el.account().cmp(from))
                    .unwrap_or_else(|idx| idx);
                if let Some(rslt) = account_rates.get_mut(idx) {
                    *rslt.rate_mut() = *offer_rate.rate();
                } else {
                    account_rates.insert(idx, AccountRate::new(from.clone(), *offer_rate.rate()))
                }
            })
        }
    }
}
