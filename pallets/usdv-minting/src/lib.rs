#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_error, decl_event, decl_module, dispatch};
use frame_system::ensure_signed;
use orml_traits::MultiCurrency;
use pallet_membership::Instance0;
use valiu_node_commons::ValiuCurrencies;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type BalanceOf<T> =
    <<T as Trait>::Currency as MultiCurrency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait Trait: pallet_membership::Trait<Instance0> {
    type Currency: MultiCurrency<Self::AccountId, CurrencyId = ValiuCurrencies>;
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_error! {
    pub enum Error for Module<T: Trait> {}
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        Mint(AccountId, ValiuCurrencies),
    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call
    where
        origin: T::Origin
    {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 0]
        pub fn mint(origin, balance: BalanceOf<T>) -> dispatch::DispatchResult{
            let provider = ensure_signed(origin)?;

            let members = pallet_membership::Module::<T, Instance0>::members();
            members.binary_search(&provider).ok().ok_or(pallet_membership::Error::<T, Instance0>::NotMember)?;

            T::Currency::deposit(ValiuCurrencies::Usdv, &provider, balance)?;
            Self::deposit_event(RawEvent::Mint(provider, ValiuCurrencies::Usdv));
            Ok(())
        }
    }
}
