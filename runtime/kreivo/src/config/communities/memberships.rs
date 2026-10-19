use super::*;

use frame_contrib_traits::memberships::GroupCollectionMemberships;
use sp_runtime::{
	str_array,
	traits::{AccountIdConversion, Convert, Verify},
};

use pallet_nfts::PalletFeatures;
use virto_common::MembershipId;

parameter_types! {
	pub MembershipsPalletFeatures: PalletFeatures = PalletFeatures::all_enabled();
	pub const MaxDeadlineDuration: BlockNumber = 12 * 30 * DAYS;
	/// Owner and administrator of every membership collection (`DEC-36`). A keyless account.
	pub const MembershipsManagerPalletId: PalletId = PalletId(*b"kv/mbmgr");
	pub MembershipsManagerAccount: AccountId = MembershipsManagerPalletId::get().into_account_truncating();
	/// Holds the memberships a community retires until they are burnt (`DEC-19`). Keyless.
	pub RetiringMembershipsAccount: AccountId = RETIRING_MEMBERSHIPS_ACCOUNT.into();
	pub RetirementHolder: Option<AccountId> = Some(RetiringMembershipsAccount::get());
	/// The attribute keys Kreivo lets `set_membership_attribute` write. None beyond what the
	/// manager writes itself.
	pub AttributeKeys: &'static [&'static [u8]] = &[];
}

/// The account that holds retiring memberships: `memberships/retiring_memberships`, 32 bytes.
pub const RETIRING_MEMBERSHIPS_ACCOUNT: [u8; 32] = str_array("memberships/retiring_memberships");

/// The account of a community, which holds the community's stock of memberships.
pub struct CommunityAccount;

impl Convert<CommunityId, AccountId> for CommunityAccount {
	fn convert(community_id: CommunityId) -> AccountId {
		Communities::community_account(&community_id)
	}
}

pub type CommunityMembershipsInstance = pallet_nfts::Instance2;

impl pallet_nfts::Config<CommunityMembershipsInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = CommunityId;
	type ItemId = MembershipId;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	// Nobody creates a collection through a call: `communities-manager` creates each community's
	// collection when it registers the community (`DEC-36`).
	#[cfg(not(feature = "runtime-benchmarks"))]
	type CreateOrigin = frame_system::EnsureNever<AccountId>;
	#[cfg(feature = "runtime-benchmarks")]
	type CreateOrigin = frame_system::EnsureSigned<AccountId>;
	type Locker = ();
	type CollectionDeposit = ();
	#[cfg(not(feature = "runtime-benchmarks"))]
	type ItemDeposit = ();
	#[cfg(feature = "runtime-benchmarks")]
	// When benchmarking, items must have a deposit cost, in order for `redeposit`
	// to work.
	type ItemDeposit = sp_core::ConstU128<CENTS>;
	type MetadataDepositBase = ();
	type AttributeDepositBase = ();
	type DepositPerByte = ();
	type StringLimit = ConstU32<256>;
	type KeyLimit = ConstU32<64>;
	type ValueLimit = ConstU32<256>;
	type ApprovalsLimit = ConstU32<20>;
	type ItemAttributesApprovalsLimit = ConstU32<30>;
	type MaxTips = ConstU32<10>;
	type MaxDeadlineDuration = MaxDeadlineDuration;
	type MaxAttributesPerCall = ConstU32<10>;
	type Features = ();
	type OffchainSignature = Signature;
	type OffchainPublic = <Signature as Verify>::Signer;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
	type WeightInfo = crate::weights::pallet_nfts_community_memberships::WeightInfo<Runtime>;
	type BlockNumberProvider = RelaychainData;
}

/// Every community's memberships live in the community's own collection (`DEC-17`).
pub type Memberships = GroupCollectionMemberships<
	CommunityMemberships,
	pallet_nfts::ItemConfig,
	CommunityAccount,
	MembershipsManagerAccount,
	RetirementHolder,
	AttributeKeys,
>;
