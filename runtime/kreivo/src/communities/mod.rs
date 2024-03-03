use super::*;

use frame_support::traits::{membership::NonFungibleAdpter, tokens::nonfungible_v2::ItemOf};
use pallet_communities::origin::EnsureCommunity;
use virto_common::{CommunityId, MembershipInfo};

pub mod governance;
pub mod memberships;

parameter_types! {
  pub const CommunityPalletId: PalletId = PalletId(*b"kv/cmtys");
	pub const MembershipsCollectionId: CollectionId = 1;
	pub const MembershipNftAttr: &'static [u8; 10] = b"membership";
}

type MembershipCollection = ItemOf<CommunityMemberships, MembershipsCollectionId, AccountId>;
type Memberships = NonFungibleAdpter<MembershipCollection, MembershipInfo, MembershipNftAttr>;

impl pallet_communities::Config for Runtime {
	type CommunityId = CommunityId;

	type CommunityMgmtOrigin = EnsureRoot<AccountId>;
	type MemberMgmtOrigin = EnsureCommunity<Self>;
	type MemberMgmt = Memberships;
	type Membership = MembershipInfo;

	type Polls = CommunityReferenda;

	type Assets = Assets;
	type Balances = Balances;
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_communities::weights::SubstrateWeight<Runtime>;

	type PalletId = CommunityPalletId;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = CommunityBenchmarkHelper;
}

#[cfg(feature = "runtime-benchmarks")]
use self::{
	governance::{CommunityReferendaInstance, CommunityTracksInstance},
	memberships::CommunityMembershipsInstance,
};

#[cfg(feature = "runtime-benchmarks")]
use ::{
	frame_benchmarking::BenchmarkError,
	frame_support::traits::{schedule::DispatchTime, tokens::nonfungible_v2::Mutate},
	frame_system::pallet_prelude::{OriginFor, RuntimeCallFor},
	pallet_communities::{
		types::{CommunityIdOf, DecisionMethodFor, MembershipIdOf, PalletsOriginOf, PollIndexOf},
		BenchmarkHelper, Origin,
	},
	pallet_nfts::Pallet as Nfts,
	pallet_referenda::{BoundedCallOf, Curve, Pallet as Referenda, TrackInfo},
	pallet_referenda_tracks::Pallet as Tracks,
	sp_runtime::Perbill,
	virto_common::MembershipId,
};

#[cfg(feature = "runtime-benchmarks")]
pub struct CommunityBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl BenchmarkHelper<Runtime> for CommunityBenchmarkHelper {
	fn community_id() -> CommunityIdOf<Runtime> {
		CommunityId::new(1)
	}
	fn community_desired_size() -> u32 {
		u32::MAX
	}
	fn community_origin(decision_method: DecisionMethodFor<Runtime>) -> OriginFor<Runtime> {
		let mut origin = Origin::<Runtime>::new(Self::community_id());
		origin.with_decision_method(decision_method.clone());
		origin.into()
	}

	fn initialize_memberships_collection() -> Result<(), BenchmarkError> {
		let collection = MembershipsCollectionId::get();
		Nfts::<Runtime, CommunityMembershipsInstance>::do_create_collection(
			collection,
			RootAccount::get(),
			RootAccount::get(),
			Default::default(),
			0,
			pallet_nfts::Event::ForceCreated {
				collection,
				owner: RootAccount::get(),
			},
		)?;

		Ok(())
	}

	fn new_membership_id(community_id: CommunityIdOf<Runtime>, index: u32) -> MembershipIdOf<Runtime> {
		MembershipId(community_id, index)
	}

	fn prepare_track_and_submit_referendum(
		origin: OriginFor<Runtime>,
		proposal_origin: PalletsOriginOf<Runtime>,
		proposal_call: RuntimeCallFor<Runtime>,
	) -> Result<PollIndexOf<Runtime>, BenchmarkError> {
		let id = Self::community_id();
		let info = TrackInfo {
			name: sp_runtime::str_array("Community"),
			max_deciding: 1,
			decision_deposit: 5,
			prepare_period: 1,
			decision_period: 5,
			confirm_period: 1,
			min_enactment_period: 1,
			min_approval: Curve::LinearDecreasing {
				length: Perbill::from_percent(100),
				floor: Perbill::from_percent(50),
				ceil: Perbill::from_percent(100),
			},
			min_support: Curve::LinearDecreasing {
				length: Perbill::from_percent(100),
				floor: Perbill::from_percent(0),
				ceil: Perbill::from_percent(100),
			},
		};

		Tracks::<Runtime, CommunityTracksInstance>::insert(RuntimeOrigin::root(), id, info, proposal_origin.clone())?;

		let bounded_call = BoundedVec::truncate_from(proposal_call.encode());
		let proposal_origin = Box::new(proposal_origin);
		let proposal = BoundedCallOf::<Runtime, CommunityReferendaInstance>::Inline(bounded_call);
		let enactment_moment = DispatchTime::After(1);

		let index = 0u32;
		Referenda::<Runtime, CommunityReferendaInstance>::submit(
			origin.clone(),
			proposal_origin,
			proposal,
			enactment_moment,
		)?;
		Referenda::<Runtime, CommunityReferendaInstance>::place_decision_deposit(origin, index)?;

		Ok(index)
	}

	fn extend_membership(
		community_id: CommunityIdOf<Runtime>,
		membership_id: MembershipIdOf<Runtime>,
	) -> Result<(), BenchmarkError> {
		let community_account = pallet_communities::Pallet::<Runtime>::community_account(&community_id);
		MembershipCollection::mint_into(&membership_id, &community_account, &Default::default(), true)?;

		Ok(())
	}
}
