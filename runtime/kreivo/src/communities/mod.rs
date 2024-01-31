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
}
