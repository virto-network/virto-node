#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod impls;
pub use impls::CommunityFeeHandler;
mod types;
use types::*;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use orml_traits::{LockIdentifier, NamedMultiReservableCurrency};
	use weights::WeightInfo;

	pub type DomainNameOf<T> = BoundedVec<u8, <T as Config>::MaxDomainNameSize>;

	#[pallet::config]
	pub trait Config: frame_system::Config + orml_payments::Config {
		/// Because this pallet emits events, it depends on the runtime's
		/// definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The type of assets this pallet can hold in payment
		type Asset: NamedMultiReservableCurrency<Self::AccountId, ReserveIdentifier = LockIdentifier>;
		/// Max length of domain name for community
		type MaxDomainNameSize: Get<u32>;
		/// Weight Config for the pallet
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn communities)]
	pub type Communities<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, BaseIndex>,
			NMapKey<Blake2_128Concat, CategoryIndex>,
			NMapKey<Blake2_128Concat, InstanceIndex>,
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
		/// Invalid format for community Id
		InvalidCommunityId,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a new community that occupies the cell id and has `domain`
		#[pallet::weight(<T as pallet::Config>::WeightInfo::register())]
		pub fn register(origin: OriginFor<T>, id: CommunityId, domain: DomainNameOf<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Communities::<T>::try_mutate((id.base, id.category, id.instance), |community| -> DispatchResult {
				ensure!(community.is_none() && id.is_valid(), Error::<T>::InvalidCommunityId);

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
