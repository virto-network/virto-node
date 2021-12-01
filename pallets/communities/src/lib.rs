#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// TODO : Add benchmarks
// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type CommunityRegistry<T> = StorageMap<
		_,
		Blake2_128Concat,
		u32, // TODO : Create communityId as speced in https://github.com/virto-network/virto-node/issues/133
		Vec<u8>,
		OptionQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new community has been created
		CommunityCreated(u32, Vec<u8>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// This dispatchable allows users to generate new community on virto network
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn register(origin: OriginFor<T>, community_id: u32, home_server_url: Vec<u8>) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;

			// reserve min deposit amount

			// Add new community to storage
			<CommunityRegistry<T>>::insert(community_id, &home_server_url);

			// Emit an event.
			Self::deposit_event(Event::CommunityCreated(community_id, home_server_url));
			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}
	}
}
