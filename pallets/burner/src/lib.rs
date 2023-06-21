#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::traits::Currency;

pub mod weights;
pub use weights::*;

use sp_runtime::traits::StaticLookup;

pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type PositiveImbalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::PositiveImbalance;
pub type NegativeImbalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;
type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*,
		traits::{
			Currency, EnsureOrigin, ExistenceRequirement::KeepAlive, Get, Imbalance, OnUnbalanced, ReservableCurrency,
			WithdrawReasons,
		},
		PalletId,
	};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it
	/// depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's
		/// definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// Handler for burning funds.
		type BurnDestination: OnUnbalanced<NegativeImbalanceOf<Self>>;

		/// Origin for burning funds.
		type BurnOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides
		/// descriptive names for event parameters. [something, who]
		Burnt {
			burnt_funds: BalanceOf<T>,
			from: T::AccountId,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke
	// state changes. These functions materialize as "extrinsics", which are often
	// compared to transactions. Dispatchable functions must be annotated with a
	// weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::burn_asset())]
		pub fn burn_asset(
			origin: OriginFor<T>,
			burn_from: AccountIdLookupOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			T::BurnOrigin::ensure_origin(origin.clone())?;

			let account_id = T::Lookup::lookup(burn_from)?;

			let mut imbalance = <PositiveImbalanceOf<T>>::zero();
			let (debit, credit) = T::Currency::pair(amount);
			imbalance.subsume(debit);
			T::BurnDestination::on_unbalanced(credit);

			// Must never be an error, but better to be safe.
			// proof: budget_remaining is account free balance minus ED;
			// Thus we can't spend more than account free balance minus ED;
			// Thus account is kept alive; qed;
			if let Err(problem) = T::Currency::settle(&account_id, imbalance, WithdrawReasons::TRANSFER, KeepAlive) {
				log::error!("Inconsistent state - couldn't settle imbalance for funds spent by treasury");
				// Nothing else to do here.
				drop(problem);
			}

			Self::deposit_event(Event::Burnt {
				burnt_funds: amount,
				from: account_id,
			});

			Ok(().into())
		}
	}
}
