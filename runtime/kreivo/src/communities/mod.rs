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
use ::{
	pallet_communities::{types::CommunityIdOf, BenchmarkHelper},
	virto_common::MembershipId,
};

#[cfg(feature = "runtime-benchmarks")]
pub struct CommunityBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl BenchmarkHelper<Runtime> for CommunityBenchmarkHelper {
	fn get_community_id() -> CommunityIdOf<Runtime> {
		CommunityId::new(1)
	}

	fn community_desired_size() -> u32 {
		u32::MAX
	}

	fn setup_members(community_id: CommunityIdOf<Runtime>) -> Result<u32, frame_benchmarking::BenchmarkError> {
		let memberships = (0..u8::MAX).map(|i| MembershipId(Self::get_community_id(), i as u32));

		let account = pallet_communities::Pallet::<Runtime>::community_account(&community_id);
		for membership in memberships {
			use frame_support::traits::tokens::nonfungible_v2::Mutate;

			MembershipCollection::mint_into(&membership, &account, &Default::default(), true)?;
		}

		Ok(u8::MAX as u32)
	}
}
