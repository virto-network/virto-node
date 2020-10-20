#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_error, decl_event, decl_module, dispatch};
use frame_system::ensure_signed;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use valiu_node_commons::Asset;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type BalanceOf<T> =
    <<T as Trait>::Currency as MultiCurrency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait Trait: pallet_membership::Trait {
    type Currency: MultiReservableCurrency<Self::AccountId, CurrencyId = Asset>;
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        Attestation(AccountId, Asset),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        MustNotBeUsdv,
        NotAProvider,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 0]
        pub fn attest(origin, asset_id: Asset, balance: BalanceOf<T>) -> dispatch::DispatchResult {
            if let Asset::Usdv = asset_id {
                return Err(Error::<T>::MustNotBeUsdv.into());
            }

            let provider = ensure_signed(origin)?;

            let members = pallet_membership::Module::<T>::members();
            members.binary_search(&provider).map_err(|_| Error::<T>::NotAProvider)?;

            T::Currency::deposit(asset_id, &provider, balance)?;
            T::Currency::reserve(asset_id, &provider, balance)?;
            Self::deposit_event(RawEvent::Attestation(provider, asset_id));
            Ok(())
        }
    }
}
