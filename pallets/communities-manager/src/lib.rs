#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use frame_contrib_traits::tracks::MutateTracks;
use frame_support::{
	pallet_prelude::*,
	traits::{nonfungibles_v2::Create as CollectionCreate, Incrementable, OriginTrait, RankedMembers},
};
use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};
use pallet_communities::{
	AccountIdOf, AssetIdOf, CommunityIdOf, DecisionMethodFor, NativeBalanceOf, Origin as CommunityOrigin,
	PalletsOriginOf, RuntimeOriginFor,
};
use pallet_nfts::CollectionConfig;
use pallet_referenda::{TrackInfo, TracksInfo};
use sp_runtime::{str_array, traits::StaticLookup};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

pub mod weights;
pub use weights::*;

type TrackInfoOf<T> = TrackInfo<NativeBalanceOf<T>, BlockNumberFor<T>>;

// The pallet macros clone every call argument, `Copy` ones included.
#[allow(clippy::clone_on_copy)]
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::DefaultNoBound;
	use parity_scale_codec::HasCompact;

	type CommunityName = BoundedVec<u8, ConstU32<25>>;

	/// Configure the pallet by specifying the parameters and types on which it
	/// depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config<RuntimeEvent: From<Event<Self>>>
		+ pallet_communities::Config<MembershipId: Incrementable + HasCompact + MaybeSerializeDeserialize>
	where
		AssetIdOf<Self>: MaybeSerializeDeserialize,
	{
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

		type RegisterOrigin: EnsureOrigin<
			OriginFor<Self>,
			Success = Option<(NativeBalanceOf<Self>, AccountIdOf<Self>, AccountIdOf<Self>)>,
		>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// A genesis community info.
	pub type GenesisCommunityOf<T> = (
		// community_id
		CommunityIdOf<T>,
		// name
		String,
		// admin
		AccountIdOf<T>,
		// maybe_decision_method
		Option<DecisionMethodFor<T>>,
		// maybe_rank
		Option<u16>,
	);

	#[pallet::genesis_config]
	#[derive(DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		/// A list of initial communities, with the most basic settings.
		pub communities: Vec<GenesisCommunityOf<T>>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			for (community_id, name, admin, maybe_decision_method, maybe_rank) in &self.communities {
				Pallet::<T>::try_register(
					None, // genesis communities are given for free :)
					*community_id,
					name.as_str(),
					admin,
					maybe_decision_method.clone(),
					None, // use the default track info. Let the community set the track later.
					*maybe_rank,
				)
				.unwrap();
			}
		}
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The community with [`CommunityId`](pallet_communities::CommunityId)
		/// has been created.
		CommunityRegistered { id: T::CommunityId },
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
			first_admin: pallet_communities::AccountIdLookupOf<T>,
			maybe_decision_method: Option<DecisionMethodFor<T>>,
			maybe_track_info: Option<TrackInfoOf<T>>,
		) -> DispatchResult {
			Self::try_register(
				T::RegisterOrigin::ensure_origin(origin)?,
				community_id,
				core::str::from_utf8(&name).map_err(|_| Error::<T>::InvalidCommunityName)?,
				&T::Lookup::lookup(first_admin)?,
				maybe_decision_method,
				maybe_track_info,
				None,
			)
		}
	}
}

impl<T: Config> Pallet<T> {
	pub(crate) fn try_register(
		maybe_deposit: Option<(NativeBalanceOf<T>, AccountIdOf<T>, AccountIdOf<T>)>,
		community_id: CommunityIdOf<T>,
		community_name: &str,
		admin: &AccountIdOf<T>,
		maybe_decision_method: Option<DecisionMethodFor<T>>,
		maybe_track_info: Option<TrackInfoOf<T>>,
		maybe_rank: Option<u16>,
	) -> DispatchResult {
		let admin_origin = frame_system::Origin::<T>::Signed(admin.clone());

		// Register first to check if community exists
		pallet_communities::Pallet::<T>::register(&admin_origin.clone().into(), &community_id, maybe_deposit)?;

		if let Some(decision_method) = maybe_decision_method {
			pallet_communities::Pallet::<T>::set_decision_method(admin_origin.into(), community_id, decision_method)?;
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
		let community_origin: RuntimeOriginFor<T> = CommunityOrigin::<T>::new(community_id).into();
		T::Tracks::insert(
			community_id,
			maybe_track_info.unwrap_or_else(|| Self::default_tack(community_name)),
			community_origin.into_caller(),
		)?;
		// Induct community at Kreivo Governance with rank 0
		T::RankedCollective::induct(&community_account)?;
		if let Some(rank) = maybe_rank {
			for _ in 0..rank {
				T::RankedCollective::promote(&community_account)?;
			}
		}

		Self::deposit_event(Event::<T>::CommunityRegistered { id: community_id });
		Ok(())
	}

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
