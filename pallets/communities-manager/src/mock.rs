use super::*;

use frame_support::{
	parameter_types,
	traits::{
		fungible::HoldConsideration, tokens::nonfungible_v2::ItemOf, AsEnsureOriginWithArg, ConstU128, ConstU16,
		ConstU32, ConstU64, EitherOf, EnsureOriginWithArg, EqualPrivilegeOnly, Footprint, OriginTrait,
	},
	PalletId,
};
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	MultiSignature,
};
pub use virto_common::{CommunityId, MembershipId};

type Block = frame_system::mocking::MockBlock<Test>;
type WeightInfo = ();

pub type AccountPublic = <MultiSignature as Verify>::Signer;
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
pub type Balance = u128;
pub type AssetId = u32;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		Balances: pallet_balances,
		Communities: pallet_communities,
		Nfts: pallet_nfts,
		System: frame_system,
		Tracks: pallet_referenda_tracks,
	}
);
