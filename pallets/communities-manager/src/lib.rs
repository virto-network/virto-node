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
	traits::{
		nonfungibles_v2::Mutate as ItemMutate,
		nonfungibles_v2::{Create as CollectionCreate, Trading},
		Incrementable, OriginTrait, RankedMembers,
	},
};
use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};
use pallet_communities::{
	types::{AccountIdOf, CommunityIdOf, DecisionMethodFor, NativeBalanceOf, PalletsOriginOf, RuntimeOriginFor},
	Origin as CommunityOrigin,
};
use pallet_nfts::{CollectionConfig, ItemConfig};
use pallet_referenda::{TrackInfo, TracksInfo};
use parity_scale_codec::Decode;
use sp_runtime::{str_array, traits::Get};

type TrackInfoOf<T> = TrackInfo<NativeBalanceOf<T>, BlockNumberFor<T>>;

#[frame_support::pallet]
pub mod pallet {
	use parity_scale_codec::HasCompact;

	use super::*;

	type CommunityName = BoundedVec<u8, ConstU32<25>>;

	/// Configure the pallet by specifying the parameters and types on which it
	/// depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_communities::Config {
		/// Because this pallet emits events, it depends on the runtime's
		/// definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type CreateCollection: CollectionCreate<
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

		type CreateMembershipsOrigin: EnsureOrigin<OriginFor<Self>>;

		type MembershipId: Parameter + Decode + Incrementable + HasCompact;

		type MembershipsManagerCollectionId: Get<CommunityIdOf<Self>>;

		type MembershipsManagerOwner: Get<AccountIdOf<Self>>;

		type CreateMemberships: ItemMutate<
				AccountIdOf<Self>,
				ItemConfig = ItemConfig,
				CollectionId = CommunityIdOf<Self>,
				ItemId = <Self as Config>::MembershipId,
			> + Trading<
				AccountIdOf<Self>,
				NativeBalanceOf<Self>,
				CollectionId = CommunityIdOf<Self>,
				ItemId = <Self as Config>::MembershipId,
			>;
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
		/// The
		MembershipsCreated {
			starting_at: <T as Config>::MembershipId,
			amount: u32,
		},
	}

	// Errors inform users that something worked or went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Community name didn't contain valid utf8 characters
		InvalidCommunityName,
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
			name: CommunityName,
			maybe_admin_origin: Option<PalletsOriginOf<T>>,
			maybe_decision_method: Option<DecisionMethodFor<T>>,
			maybe_track_info: Option<TrackInfoOf<T>>,
			// _maybe_first_member: Option<AccountIdLookupOf<T>>,
		) -> DispatchResult {
			let maybe_deposit = T::CreateOrigin::ensure_origin(origin)?;

			let community_name = core::str::from_utf8(&name).map_err(|_| Error::<T>::InvalidCommunityName)?;
			let community_origin: RuntimeOriginFor<T> = CommunityOrigin::<T>::new(community_id).into();
			let admin_origin = maybe_admin_origin.unwrap_or(community_origin.clone().into_caller());
			// Register first to check if community exists
			pallet_communities::Pallet::<T>::register(&admin_origin, &community_id, maybe_deposit)?;

			if let Some(decision_method) = maybe_decision_method {
				pallet_communities::Pallet::<T>::set_decision_method(
					admin_origin.clone().into(),
					community_id,
					decision_method,
				)?;
			}

			let community_account = pallet_communities::Pallet::<T>::community_account(&community_id);

			// Create memberships collection for community
			T::CreateCollection::create_collection_with_id(
				community_id,
				&community_account,
				&community_account,
				&CollectionConfig {
					settings: Default::default(),
					max_supply: None,
					mint_settings: Default::default(),
				},
			)?;

			// Create governance track for community
			T::Tracks::insert(
				community_id,
				maybe_track_info.unwrap_or_else(|| Self::default_tack(community_name)),
				community_origin.into_caller(),
			)?;
			// Induct community at Kreivo Governance with rank 0
			T::RankedCollective::induct(&community_account)?;

			Self::deposit_event(Event::<T>::CommunityRegistered { id: community_id });
			Ok(())
		}

		#[pallet::call_index(1)]
		pub fn create_memberships(
			origin: OriginFor<T>,
			#[pallet::compact] amount: u32,
			#[pallet::compact] starting_at: <T as Config>::MembershipId,
			#[pallet::compact] price: NativeBalanceOf<T>,
		) -> DispatchResult {
			T::CreateMembershipsOrigin::ensure_origin(origin)?;

			let mut id = starting_at.clone();
			let mut minted = 0u32;
			for _ in 0..amount {
				T::CreateMemberships::mint_into(
					&T::MembershipsManagerCollectionId::get(),
					&id,
					&T::MembershipsManagerOwner::get(),
					&Default::default(),
					true,
				)?;

				T::CreateMemberships::set_price(
					&T::MembershipsManagerCollectionId::get(),
					&id,
					&T::MembershipsManagerOwner::get(),
					Some(price),
					None,
				)?;
				if let Some(next_id) = id.increment() {
					id = next_id;
					minted += 1;
				} else {
					break;
				}
			}

			Self::deposit_event(Event::<T>::MembershipsCreated {
				starting_at,
				amount: minted,
			});
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn default_tack(name: &str) -> TrackInfoOf<T> {
			use sp_runtime::Perbill;
			TrackInfo {
				name: str_array(name),
				max_deciding: 1,
				decision_deposit: 0u8.into(),
				prepare_period: 1u8.into(),
				decision_period: u8::MAX.into(),
				confirm_period: 1u8.into(),
				min_enactment_period: 1u8.into(),
				min_approval: pallet_referenda::Curve::LinearDecreasing {
					length: Perbill::from_percent(100),
					floor: Perbill::from_percent(50),
					ceil: Perbill::from_percent(100),
				},
				min_support: pallet_referenda::Curve::LinearDecreasing {
					length: Perbill::from_percent(100),
					floor: Perbill::from_percent(0),
					ceil: Perbill::from_percent(50),
				},
			}
		}
	}
}
