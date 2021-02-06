#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
mod crypto;
#[cfg(any(feature = "runtime-benchmarks", test))]
mod mock;
mod module_impl;
mod offchain_error;
#[cfg(test)]
mod tests;
mod weights;

use alloc::vec::Vec;
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, traits::Get,
};
use frame_system::{
    ensure_none, ensure_signed,
    offchain::{AppCrypto, SendTransactionTypes, SigningTypes},
};
use offchain_error::*;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use sp_arithmetic::traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};
use sp_runtime::traits::Zero;
use vln_commons::{AccountRate, Asset, Destination, OfferRate, PairPrice};

pub use crypto::*;
pub use module_impl::module_impl_offchain::*;
pub use weights::*;

type AccountRateTy<T> = AccountRate<<T as frame_system::Trait>::AccountId, Balance<T>>;
type Balance<T> =
    <<T as Trait>::Collateral as MultiCurrency<<T as frame_system::Trait>::AccountId>>::Balance;
type OfferRateTy<T> = OfferRate<Balance<T>>;
type LiquidityMembers = pallet_membership::DefaultInstance;

pub trait Trait:
    SendTransactionTypes<Call<Self>> + SigningTypes + pallet_membership::Trait<LiquidityMembers>
where
    Balance<Self>: LiquidityProviderBalance,
{
    type Asset: MultiCurrency<Self::AccountId, Balance = Balance<Self>, CurrencyId = Asset>;
    type Collateral: MultiReservableCurrency<Self::AccountId, CurrencyId = Asset>;
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type OffchainAuthority: AppCrypto<Self::Public, Self::Signature>;
    type OffchainUnsignedInterval: Get<Self::BlockNumber>;
    type WeightInfo: WeightInfo;
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        MustBeCollateral,
        NoFunds,
        TransferMustBeGreaterThanZero,
        DestinationNotSupported
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        Balance = Balance<T>,
    {
        Attestation(AccountId, Asset),
        Members(Vec<AccountId>),
        Transfer(AccountId, AccountId, Balance),
    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call
    where
        origin: T::Origin
    {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = T::WeightInfo::attest()]
        pub fn attest(
            origin,
            asset: Asset,
            balance: Balance<T>,
            offer_rates: Vec<OfferRateTy<T>>
        ) -> DispatchResult
        {
            match asset {
                Asset::Btc | Asset::Cop | Asset::Usdv | Asset::Ves => {
                    Err(crate::Error::<T>::MustBeCollateral.into())
                },
                Asset::Collateral(collateral) => {
                    let who = ensure_signed(origin)?;
                    Self::update_account_rates(&who, asset, offer_rates);
                    Self::do_attest(who.clone(), Asset::Usdv, balance)?;
                    T::Collateral::deposit(collateral.into(), &who, balance)?;
                    T::Collateral::reserve(collateral.into(), &who, balance)?;
                    Self::deposit_event(RawEvent::Attestation(who, collateral.into()));
                    Ok(())
                }
            }
        }

        #[weight = T::WeightInfo::members()]
        pub fn members(origin) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            let members = pallet_membership::Module::<T, LiquidityMembers>::members();
            Self::deposit_event(RawEvent::Members(members));
            Ok(())
        }

        #[weight = T::WeightInfo::submit_pair_prices()]
        pub fn submit_pair_prices(
            origin,
            pair_prices: Vec<PairPrice<Balance<T>>>,
            _signature: T::Signature,
        ) -> DispatchResult {
            ensure_none(origin)?;
            <PairPrices<T>>::mutate(|old_pair_prices| {
                if Self::incoming_pair_prices_are_valid(&pair_prices) {
                    old_pair_prices.clear();
                    old_pair_prices.extend(pair_prices);
                }
                else {
                    debug::error!("Invalid pair prices");
                }
            });
            let current_block = <frame_system::Module<T>>::block_number();
            <NextUnsignedAt<T>>::put(current_block + T::OffchainUnsignedInterval::get());
            Ok(())
        }

        #[weight = T::WeightInfo::transfer()]
        pub fn transfer(
            origin,
            to: Destination<<T as frame_system::Trait>::AccountId>,
            to_amount: Balance<T>,
        ) -> DispatchResult
        {
            // process transfer based on the destination type
            match to {
                // preform onchain transfer for vln address
                Destination::Vln(to_address) => {
                    let from = ensure_signed(origin)?;
                    if to_amount.is_zero() {
                        return Err(crate::Error::<T>::TransferMustBeGreaterThanZero.into());
                    }
                    Self::transfer_evenly(from, to_address, to_amount)?;
                    Ok(())
                }
                // skip all other destinations for now
                _ => Err(crate::Error::<T>::DestinationNotSupported.into())
            }
        }

        #[weight = T::WeightInfo::update_offer_rates()]
        pub fn update_offer_rates(
            origin,
            asset: Asset,
            offer_rates: Vec<OfferRateTy<T>>
        ) -> DispatchResult
        {
            let who = ensure_signed(origin)?;
            Self::update_account_rates(&who, asset, offer_rates);
            Ok(())
        }

        fn offchain_worker(block_number: T::BlockNumber) {
            if let Err(e) = Self::fetch_pair_prices_and_submit_tx(block_number) {
                debug::error!("Offchain error: {}", e);
            }
        }
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as LiquidityProviderStorage {
        pub AccountRates get(fn account_rates):
            double_map hasher(twox_64_concat) Asset,
            hasher(twox_64_concat) Asset => Vec<AccountRateTy<T>>;

        pub NextUnsignedAt get(fn next_unsigned_at): T::BlockNumber;

        pub PairPrices get(fn prices): Vec<PairPrice<Balance<T>>>
    }
}

pub trait LiquidityProviderBalance:
    CheckedAdd + CheckedDiv + CheckedMul + CheckedSub + From<u32>
{
}

impl<T> LiquidityProviderBalance for T where
    T: CheckedAdd + CheckedDiv + CheckedMul + CheckedSub + From<u32>
{
}
