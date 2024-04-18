#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

use fc_traits_tracks::MutateTracks;
use frame_support::{
	pallet_prelude::*,
	traits::{nonfungibles_v2::Create, OriginTrait, RankedMembers},
};
use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};
use pallet_communities::{
	types::{
		AccountIdLookupOf, AccountIdOf, CommunityIdOf, DecisionMethodFor, NativeBalanceOf, PalletsOriginOf,
		RuntimeOriginFor,
	},
	Origin as CommunityOrigin,
};
use pallet_nfts::{CollectionConfig, MintSettings, MintType::Issuer};
use pallet_referenda::{TrackInfo, TracksInfo};

type TrackInfoOf<T> = TrackInfo<NativeBalanceOf<T>, BlockNumberFor<T>>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it
	/// depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_communities::Config {
		/// Because this pallet emits events, it depends on the runtime's
		/// definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type CreateCollection: Create<
			AccountIdOf<Self>,
			CollectionConfig<NativeBalanceOf<Self>, BlockNumberFor<Self>, CommunityIdOf<Self>>,
			CollectionId = CommunityIdOf<Self>,
		>;

		type Tracks: TracksInfo<NativeBalanceOf<Self>, BlockNumberFor<Self>>
			+ MutateTracks<
				NativeBalanceOf<Self>,
				BlockNumberFor<Self>,
				Id = CommunityIdOf<Self>,
				RuntimeOrigin = PalletsOriginOf<Self>,
			>;

		type RankedCollective: RankedMembers<AccountId = AccountIdOf<Self>>;

		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;

		// #[cfg(feature = "runtime-benchmarks")]
		// type BenchmarkHelper: BenchmarkHelper<Self>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The community with [`CommmunityId`](pallet_communities::CommunityId)
		/// has been created.
		CommunityRegistered { id: T::CommunityId },
	}

	// Errors inform users that something worked or went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// It was not possible to register the community
		CannotRegister,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke
	// state changes. These functions materialize as "extrinsics", which are often
	// compared to transactions. Dispatchable functions must be annotated with a
	// weight and must return a DispatchResult.
	#[pallet::call(weight(<T as Config>::WeightInfo))]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		pub fn register(
			origin: OriginFor<T>,
			community_id: CommunityIdOf<T>,
			track_info: TrackInfoOf<T>,
			maybe_admin_origin: Option<PalletsOriginOf<T>>,
			maybe_decision_method: Option<DecisionMethodFor<T>>,
			_maybe_first_member: Option<AccountIdLookupOf<T>>,
		) -> DispatchResult {
			let maybe_deposit = T::CreateOrigin::ensure_origin(origin)?;

			// This implies depositing (Deposit (Hold | -> Treasury))
			let community_origin: RuntimeOriginFor<T> = CommunityOrigin::<T>::new(community_id).into();
			let admin_origin = maybe_admin_origin.unwrap_or(community_origin.clone().into_caller());

			if let Some(decision_method) = maybe_decision_method {
				pallet_communities::Pallet::<T>::set_decision_method(
					admin_origin.clone().into(),
					community_id,
					decision_method,
				)?;
			}

			pallet_communities::Pallet::<T>::register(&admin_origin, &community_id, maybe_deposit)?;

			let community_account = pallet_communities::Pallet::<T>::community_account(&community_id);

			// Create memberships collection for community
			T::CreateCollection::create_collection_with_id(
				community_id,
				&community_account,
				&community_account,
				&CollectionConfig {
					settings: Default::default(),
					max_supply: None,
					mint_settings: MintSettings {
						mint_type: Issuer,
						price: None,
						start_block: None,
						end_block: None,
						default_item_settings: Default::default(),
					},
				},
			)?;

			// Create governance track for community
			T::Tracks::insert(community_id, track_info, community_origin.into_caller())?;

			// Induct community at Kreivo Governance with rank 1
			T::RankedCollective::induct(&community_account)?;
			T::RankedCollective::promote(&community_account)?;

			Self::deposit_event(Event::<T>::CommunityRegistered { id: community_id });

			Ok(())
		}
	}
}
