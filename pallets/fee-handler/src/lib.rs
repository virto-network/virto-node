#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{pallet_prelude::*, traits::fungible};

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub type BalanceOf<T> = <<T as Config>::NativeBalance as fungible::Inspect<
	<T as frame_system::Config>::AccountId,
>>::Balance;

use frame_support::dispatch::GetDispatchInfo;
use sp_runtime::traits::Convert;

#[frame_support::pallet]
pub mod pallet {

	use crate::*;
	use frame_support::{
		dispatch::{RawOrigin},
		traits::fungible::{Inspect, InspectFreeze, MutateFreeze},
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{CheckedSub, Dispatchable};
	use sp_std::prelude::*;
	

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type to access the Balances Pallet.
		type NativeBalance: fungible::Inspect<Self::AccountId>
			+ fungible::Mutate<Self::AccountId>
			+ fungible::hold::Inspect<Self::AccountId>
			+ fungible::hold::Mutate<Self::AccountId>
			+ fungible::freeze::Inspect<Self::AccountId, Id = ()>
			+ fungible::freeze::Mutate<Self::AccountId>
			+ fungible::MutateFreeze<Self::AccountId>;

		/// A type representing all calls available in your runtime.
		// #[pallet::no_default_bounds]
		type RuntimeCall: Parameter
			+ Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
			+ GetDispatchInfo;

		// This will be compared with the available weight/credit a user has when want to perform a
		// free tx
		#[pallet::constant]
		type MaxCreditUserCanUsePerRound: Get<u64>;

		// Global constant to define the max credit can be used in an "era"
		#[pallet::constant]
		type MaxGlobalTotalCreditToUsePerRound: Get<u64>;

		type ConvertBalanceToWeight: Convert<BalanceOf<Self>, Weight>;

		// Here I want to include a convertion rate factor. Pending to be confirmed if this is a
		// good idea
		#[pallet::constant]
		type ConvertionRateLockedToWeight: Get<u32>;

		// Number of blocks which represent an era, this means, after how many blocks the credits
		// are reseted.
		#[pallet::constant]
		type BlocksOfEra: Get<BlockNumberFor<Self>>;

		// type BaseWeightUnit: Get<Weight>;
	}

	/* ----- Structures ----- */
	// Johan delete/modify this comment later to put something really awesome and funny
	// Here goes a struct which is going to store:
	// the account id, with this i will get the balance, even the locked balance for free tx, so I
	// can know how much credit this account has the block id when was locked the tokens to get
	// credit, this will allow to compare if its from the current "era" or from previous one
	// the used credit, this way I can compare with the maximum theorical locked->credit the account
	// has
	#[derive(Clone, Debug, PartialEq, TypeInfo, Encode, Decode, MaxEncodedLen, Default)]
	#[scale_info(skip_type_params(T))]
	pub struct ZeroFeesLock<T: Config> {
		block_id_to_compute_era: BlockNumberFor<T>,
		available_credit: Weight,
		used_credit: Weight,
	}

	/* ----- Storages ----- */
	/// Used to compare later with MaxTotalCreditCanBeUsedPerRound
	/// On finalize delete so is not really stored in the storage
	#[pallet::storage]
	pub type CurrentTotalCreditUsed<T: Config> = StorageValue<_, Weight, ValueQuery>;

	// // Johan here goes the storage for the storage of a ZeroFeesLock related to the accountid of
	// the user
	#[pallet::storage]
	type FreeTx<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ZeroFeesLock<T>>;

	#[pallet::storage]
	pub type CurrentEra<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	/* ----- Events ----- */
	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Success on free transaction
		TxSuccess,

		/// Epic fail on free transaction
		EpicFailOnFreeTx,

		// /// In case I have moretime implement this maybe using proxy pallet(?)
		// AccountsSponsored { time: BlockNumberFor<T>, who: BoundedVec<T::AccountId,
		// ConstU32<100>> },
		/// Triggered whe someone lock balance succesfully
		LockedDatBalance { time: BlockNumberFor<T>, who: T::AccountId, amount: BalanceOf<T> },

		/// Event when someone unlock balance succesfully
		UnlockedDatBalance { time: BlockNumberFor<T>, who: T::AccountId, amount: BalanceOf<T> },

		/// Fail when someone try to lock balance
		FailLockingDatBalance { who: BoundedVec<T::AccountId, ConstU32<100>> },

		/// Fail when someone try to unlock balance
		FailUnlockingDatBalance { who: BoundedVec<T::AccountId, ConstU32<100>> },
	}

	/* ----- Errors ----- */
	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error due to maximum total global credit used
		MaximumGlobalCreditReached,

		/// Error due to maximum credit per user used
		MaximumUserCreditReached,

		/// Error due to not enough credits to dispatch a free tx
		NotEnoughCredits,

		/// Not enough free balance to lock
		NotEnoughFreeBalanceToLock,

		/// Error during unlocking
		NotEnoughBalanceToUnlock,

		/// User doesn't exist
		NotFoundUser,

		/// Weight overflow
		WeightOverflow,

		/// Weight underflow
		WeightUnderflow,

		/// Free credit overflow Max global usage of credits
		OverflowMaxGlobalCredits,

		/// Free credit overflow Max user usage of credits
		OverflowMaxUserCredits,
	}

	/* ----- Calls ----- */
	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that charges no fee if successful.
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::default())]
		pub fn free_tx(
			origin: OriginFor<T>,
			call: Box<<T as Config>::RuntimeCall>,
		) -> DispatchResultWithPostInfo {
			// Get the signer
			let who = ensure_signed(origin)?;
			let weight: Weight = call.get_dispatch_info().weight;

			let max_cred_user_can_per_round =
				Weight::from_all(T::MaxCreditUserCanUsePerRound::get());
			let max_global_credit_can_be_used =
				Weight::from_all(T::MaxGlobalTotalCreditToUsePerRound::get());

			// Get the current era
			// JohanCheck non zero division
			let current_era = CurrentEra::<T>::get() / (T::BlocksOfEra::get());

			// Get the FreeTx data structure of the user
			// The error will never be triggered (?)
			let current_freetx_st_balance =
				FreeTx::<T>::get(&who).ok_or(Error::<T>::NotFoundUser)?;

			// Check if the current era is bigger than the era in current_freetx_st_balance
			// If current_era is bigger set credit to maximum truncated
			let frozen_balance = T::NativeBalance::balance_frozen(&(), &who);

			// Check if a new era passed since last freetx
			let is_new_era = current_era > current_freetx_st_balance.block_id_to_compute_era;

			// Assign the current_available_weight
			let current_available_weight = if is_new_era {
				T::ConvertBalanceToWeight::convert(frozen_balance)
			} else {
				current_freetx_st_balance.available_credit
			};

			ensure!(
				current_freetx_st_balance.available_credit.all_gt(weight),
				Error::<T>::NotEnoughCredits
			);

			let new_current_user_used = current_freetx_st_balance
				.used_credit
				.checked_add(&weight)
				.ok_or(Error::<T>::MaximumUserCreditReached)?;
			let new_current_global_used = CurrentTotalCreditUsed::<T>::get()
				.checked_add(&weight)
				.ok_or(Error::<T>::MaximumGlobalCreditReached)?;

			// Check maximum values before store
			ensure!(
				max_global_credit_can_be_used.all_gt(new_current_global_used),
				Error::<T>::OverflowMaxGlobalCredits
			);
			ensure!(
				max_cred_user_can_per_round.all_gt(new_current_user_used),
				Error::<T>::OverflowMaxUserCredits
			);

			// Insert new values to storage
			CurrentTotalCreditUsed::<T>::set(new_current_global_used);
			FreeTx::<T>::insert(
				&who,
				ZeroFeesLock {
					block_id_to_compute_era: current_era,
					used_credit: new_current_user_used,
					available_credit: current_available_weight,
				},
			);

			let res = call.dispatch(RawOrigin::Signed(who).into());
			res.map(|_| ()).map_err(|e| e.error)?;
			Ok(Pays::No.into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(Weight::default())]
		pub fn lock_balance_for_free_tx(
			origin: OriginFor<T>,
			amount_to_lock: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			// Get the current balance and return error if it doesn't exist
			let current_balance = T::NativeBalance::total_balance(&who);

			// Johan transform this two calls into a function to return frozen and free balance
			// Get the free balance of &who
			let frozen_balance = T::NativeBalance::balance_frozen(&(), &who);

			// Get new free balance
			let free_balance = current_balance
				.checked_sub(&frozen_balance)
				.ok_or(Error::<T>::NotEnoughFreeBalanceToLock)?;

			// New frozen balance
			ensure!(free_balance >= amount_to_lock, Error::<T>::NotEnoughFreeBalanceToLock);

			// Freeze the new amount
			T::NativeBalance::extend_freeze(&(), &who, amount_to_lock)?;

			// Get the current era of the block
			let current_era = CurrentEra::<T>::get();

			// Get the FreeTx data structure of the user
			let mut current_freetx_st_balance = FreeTx::<T>::get(&who).unwrap_or(ZeroFeesLock {
				block_id_to_compute_era: current_era,
				available_credit: Weight::default(),
				used_credit: Weight::default(),
			});

			current_freetx_st_balance.available_credit =
				if current_era > current_freetx_st_balance.block_id_to_compute_era {
					T::ConvertBalanceToWeight::convert(frozen_balance)
				} else {
					let remaining_credit = current_freetx_st_balance.available_credit;
					let new_credit = T::ConvertBalanceToWeight::convert(amount_to_lock);
					remaining_credit.checked_add(&new_credit).ok_or(Error::<T>::WeightOverflow)?
				};

			FreeTx::<T>::insert(&who, current_freetx_st_balance);

			// Notify with an event
			// Deposit a basic event.
			Self::deposit_event(Event::LockedDatBalance {
				time: <frame_system::Pallet<T>>::block_number(),
				who,
				amount: amount_to_lock,
			});

			Ok(Pays::No.into())
		}

		/// An example of re-dispatching a call
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::default())]
		pub fn unlock_balance(
			origin: OriginFor<T>,
			amount_to_unlock: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			// Get the free balance of &who
			let frozen_balance = T::NativeBalance::balance_frozen(&(), &who);

			// New frozen balance
			ensure!(frozen_balance >= amount_to_unlock, Error::<T>::NotEnoughBalanceToUnlock);

			// Get new free balance
			let frozen_balance = frozen_balance
				.checked_sub(&amount_to_unlock)
				.ok_or(Error::<T>::NotEnoughBalanceToUnlock)?;

			// Freeze the new frozen_balance amount
			T::NativeBalance::set_freeze(&(), &who, frozen_balance)?;

			// Notify with an event
			// Deposit a basic event.
			Self::deposit_event(Event::UnlockedDatBalance {
				time: <frame_system::Pallet<T>>::block_number(),
				who,
				amount: amount_to_unlock,
			});
			Ok(Pays::No.into())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_all(100_000_000_000/10))]
		pub fn fake_function_with_high_weight(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin);
			// Evento aca
			// Self::deposit_eve
			Ok(())
		}
	}

	/* ----- Hooks ----- */
	/* Johan this is something to do in case of having time *
	 *  Here I should use only on_finalize to clean the storage if something is not necessary to
	 *  be stored, eg, some global values ... I guess */

	// This pallet implements the [`frame_support::traits::Hooks`] trait to define some logic to
	// execute in some context.
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		// `on_initialize`
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			/* Manage the reset or not of the global credit */
			// Get the current era value and change it in case of increased
			let current_era = CurrentEra::<T>::get();
			let possible_next_era = n / T::BlocksOfEra::get();
			if current_era < possible_next_era {
				// Calculate the era this return the integer of divide current_block_number
				CurrentEra::<T>::set(possible_next_era);
				// Clean
				CurrentTotalCreditUsed::<T>::set(Weight::default());
			}
			// Return the weight consumed by this operation
			Weight::default()
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Get the weight of a call.
	pub fn call_weight(call: <T as Config>::RuntimeCall) -> Weight {
		call.get_dispatch_info().weight
	}

	pub fn balance_to_weight(amount: BalanceOf<T>) -> Weight {
		T::ConvertBalanceToWeight::convert(amount)
	}

	pub fn weight_diff(w1: Weight, w2: Weight) -> Option<Weight> {
		w1.checked_sub(&w2)
	}

	pub fn is_call_allowed(call_weight: Weight, user_credits: Weight) -> bool {
		call_weight.all_lt(user_credits)
	}
}
