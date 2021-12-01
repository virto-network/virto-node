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
	use orml_traits::{MultiCurrency, MultiReservableCurrency};
	use virto_primitives::{
		CommunityId, CommunityIdLower, CommunityIdRes, CommunityIdUpper, HomeServerUrl,
	};

	type BalanceOf<T> =
		<<T as Config>::Asset as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
	type AssetIdOf<T> =
		<<T as Config>::Asset as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// the type of assets this pallet can hold in payment
		type Asset: MultiReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Community Registry Storage
	pub type CommunityRegistry<T> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, CommunityIdLower>,
			NMapKey<Blake2_128Concat, CommunityIdUpper>,
			NMapKey<Blake2_128Concat, CommunityIdRes>,
		),
		HomeServerUrl,
		OptionQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new community has been created
		CommunityCreated(CommunityId, Vec<u8>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Community Id should be unique
		CommunityIdAlreadyExists,
		/// Reserve amount failed
		DepositReserveFailed,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// This dispatchable allows users to generate new community on virto network
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn register(
			origin: OriginFor<T>,
			community_id: CommunityId,
			home_server_url: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// ensure communityId is unique
			ensure!(
				!CommunityRegistry::<T>::contains_key((
					community_id.lower,
					community_id.upper,
					community_id.res
				)),
				Error::<T>::CommunityIdAlreadyExists
			);

			// TODO : reserve the community creation deposit from creator
			//T::Asset::reserve(asset, &who, 1)?;

			// Add new community to storage
			<CommunityRegistry<T>>::insert(
				(community_id.lower, community_id.upper, community_id.res),
				&home_server_url,
			);

			// Emit an event.
			Self::deposit_event(Event::CommunityCreated(community_id, home_server_url));
			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}
	}
}
