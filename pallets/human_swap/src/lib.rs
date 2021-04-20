#![allow(
    clippy::unused_unit,
    unused_qualifications,
    missing_debug_implementations
)]
#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub enum SwapState<Proof> {
	Created,
	Rejected,
	Completed(Proof),
}

pub struct SwapDetails<Proof> {
    sender : Destination,
    recipent : Destination,
    human : RateProvider,
    amount : Balance,
    from_currency : Currency,
    to_currency : Currency,
    proof : Proof,
    state : SwapState
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type SwapHandler : ValiuSwapHandler;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn something)]
    pub type Something<T> = StorageValue<_, u32>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    //#[pallet::metadata(T::AccountId = "AccountId")]
    pub enum Event<T: Config> {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        SomethingStored(u32, T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        SomeError,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// An example dispatchable that takes a singles value as a parameter, writes the value to
        /// storage and emits an event. This function must be dispatched by a signed extrinsic.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn swap_with(who: Origin, human: RateProvider, amount: Balance, from_currency: Currency, to_currency: Currency) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // handle differently based on onchain/offchain sender/recipent - maybe can handle this is swaphandler?
            match from_currency {
                Fiat() => SwapHandler::create_cash_in(),
                Collateral() => SwapHandler::create_cash_out()
            }

            // store new swap with state SwapState::Created
            // emit event with swap details
            Ok(().into())
        }

        // This should be called by the SwapDetails::Human??
        pub fn complete_swap(who: Origin, swap_id: SwapId, proof : Option<Proof>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // sanity checks

            let swap = getSwapFromStorage();
            match swap.recipent {
                Destination::offchain => SwapHandler::complete_cash_in(who, amount, destination)
                Destination::onchain => SwapHandler::complete_cash_out(who, proof)
            }

            // update storage of swap with state SwapState::Completed(proof)
            // emit event with swap details
            Ok(().into())
        }
    }
}
