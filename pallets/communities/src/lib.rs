#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

use frame_support::RuntimeDebug;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound(T: pallet::Config))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Community<T: pallet::Config> {
	// TODO : Maybe every community can configure an asset?
	pub controller: T::AccountId,
	pub population: PopulationOf<T>,
	pub domain_name: DomainNameOf<T>,
}

// TODO : Use better representative names
pub type H3CellIndex = u32;
pub type H3CellRes = u32;
pub type H3CellValue = u32;

#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// TODO : Use better representative names
pub struct CommunityId {
	index: H3CellIndex,
	res: H3CellRes,
	value: H3CellValue,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Currency};
	use frame_system::pallet_prelude::*;
	use orml_traits::{arithmetic::Zero, LockIdentifier, MultiCurrency, NamedMultiReservableCurrency};

	pub type PopulationOf<T> = BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxPopulation>;

	pub type DomainNameOf<T> = BoundedVec<u8, <T as Config>::MaxDomainNameSize>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's
		/// definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The type of assets this pallet can hold in payment
		type Asset: NamedMultiReservableCurrency<Self::AccountId, ReserveIdentifier = LockIdentifier>;
		/// Max population allowed in a cell
		type MaxPopulation: Get<u32>;
		/// Max length of domain name for community
		type MaxDomainNameSize: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn communities)]
	pub type Communities<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, H3CellIndex>,
			NMapKey<Blake2_128Concat, H3CellRes>,
			NMapKey<Blake2_128Concat, H3CellValue>,
		),
		Community<T>,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new community registered
		CommunityRegistered(CommunityId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Cell is already occupied
		CellAlreadyOccupied,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a new community that occupies the cell id and has `domain`
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn register(origin: OriginFor<T>, id: CommunityId, domain: DomainNameOf<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Communities::<T>::try_mutate((id.index, id.res, id.value), |community| -> DispatchResult {
				ensure!(community.is_none(), Error::<T>::CellAlreadyOccupied);

				let new_community = Community {
					controller: who,
					population: Default::default(),
					domain_name: domain,
				};

				// TODO : Reserve a fee for registering community?
				*community = Some(new_community);

				Ok(())
			})?;

			// Emit an event.
			Self::deposit_event(Event::CommunityRegistered(id));

			Ok(().into())
		}

		/// Join an existing community with given communityid
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn join(_origin: OriginFor<T>, _id: CommunityId) -> DispatchResultWithPostInfo {
			todo!()
		}

		/// Leave an existing community with given communityid
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn exit(_origin: OriginFor<T>, _id: CommunityId) -> DispatchResultWithPostInfo {
			todo!()
		}
	}
}
