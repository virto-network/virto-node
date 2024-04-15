#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
pub(crate) mod mock;
// #[cfg(test)]
// mod tests;

pub mod weights;
pub use weights::*;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::OriginFor;
use pallet_communities::types::{
	AccountIdLookupOf, CommunityIdOf, ConstSizedField, DecisionMethodFor, PalletsOriginOf,
};
use pallet_referenda_tracks::TrackInfoOf;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it
	/// depends.
	#[pallet::config]
	pub trait Config<I: 'static = ()>:
		frame_system::Config + pallet_communities::Config + pallet_nfts::Config<I> + pallet_referenda_tracks::Config<I>
	{
		/// Because this pallet emits events, it depends on the runtime's
		/// definition of an event.
		type RuntimeEvent: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;

		type MembershipsMgmtCollection: Get<<Self as pallet_nfts::Config<I>>::CollectionId>;

		// #[cfg(feature = "runtime-benchmarks")]
		// type BenchmarkHelper: BenchmarkHelper<Self>;
	}

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(_);

	#[pallet::origin]
	pub struct Origin<T, I = ()>(PhantomData<(T, I)>);

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// The community with [`CommmunityId`](pallet_communities::CommunityId)
		/// has been created.
		CommunityCreated {
			id: T::CommunityId,
			origin: PalletsOriginOf<T>,
		},
	}

	// Errors inform users that something worked or went wrong.
	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// It was not possible to register the community
		CannotRegister,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke
	// state changes. These functions materialize as "extrinsics", which are often
	// compared to transactions. Dispatchable functions must be annotated with a
	// weight and must return a DispatchResult.
	#[pallet::call(weight(<T as Config>::WeightInfo))]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::call_index(0)]
		pub fn register(
			origin: OriginFor<T>,
			first_member: AccountIdLookupOf<T>,
			maybe_id: Option<CommunityIdOf<T>>,
			maybe_admin_origin: Option<PalletsOriginOf<T>>,
			maybe_decision_method: Option<DecisionMethodFor<T>>,
			maybe_track_info: Option<TrackInfoOf<T, I>>,
		) -> DispatchResult {
			// This implies depositing (Deposit (Hold | -> Treasury))
			let lowest_community_id_available = todo!("calculate this");
			let community_id = maybe_id.unwrap_or(lowest_community_id_available);
			let community_origin = pallet_communities::Origin::<T>::new(community_id);
			let admin_origin = maybe_admin_origin.unwrap_or(community_origin);

			pallet_communities::Pallet::<T>::register(community_id, admin_origin)?;

			if let Some(decision_method) = maybe_decision_method {
				pallet_communities::Pallet::<T>::set_decision_method(admin_origin, decision_method)?;
			}

			let community_account = pallet_communities::Pallet::<T>::community_account(&community_id);

			// Create the community memberships collection.
			pallet_nfts::Pallet::<T, I>::create(Origin.into(), community_id, community_account)?;

			// Mint the first membership for the community]
			let item_id = todo!("calculate this");
			pallet_nfts::Pallet::<T, I>::mint_into(
				Origin.into(),
				T::MembershipsMgmtCollection::get(),
				item_id,
				community_account,
			)?;

			pallet_communities::Pallet::<T>::add_member(admin_origin, first_member)?;

			pallet_referenda_tracks::Pallet::<T, I>::insert(
				Origin.into(),
				maybe_track_info.unwrap_or(TrackInfoOf::<T, I> {
					// TODO: add missing fields
				}),
			)?;

			Self::deposit_event(Event::<T, I>::CommunityCreated {
				id: community_id,
				origin: admin_origin,
			});

			Ok(())
		}

		#[pallet::call_index(1)]
		pub fn set_metadata(
			origin: OriginFor<T>,
			name: Option<ConstSizedField<64>>,
			description: Option<ConstSizedField<256>>,
			url: Option<ConstSizedField<256>>,
		) -> DispatchResult {
			todo!("implement this")
			// Deposit (Hold)
			// Communities::set_metadata(name, description, url)
		}

		#[pallet::call_index(2)]
		pub fn configure_track(origin: OriginFor<T>, track_details: TrackInfoOf<T>) -> DispatchResult {
			todo!("implement")
		}
	}
}
