use super::*;

use frame_support::traits::EitherOf;
use frame_system::EnsureRootWithSuccess;
use virto_common::MembershipId;

pub type CommunityMembershipsInstance = pallet_nfts::Instance2;

// From https://github.com/polkadot-fellows/runtimes/blob/main/system-parachains/asset-hubs/asset-hub-kusama/src/lib.rs#L810
impl pallet_nfts::Config<CommunityMembershipsInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type CollectionId = CollectionId;
	type ItemId = MembershipId;

	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	// Ensure only root is allowed to executing `create` calls
	type CreateOrigin =
		AsEnsureOriginWithArg<EitherOf<EnsureRootWithSuccess<AccountId, RootAccount>, EnsureSigned<AccountId>>>;
	type Locker = ();

	type CollectionDeposit = NftsCollectionDeposit;
	type ItemDeposit = NftsItemDeposit;
	type MetadataDepositBase = NftsMetadataDepositBase;
	type AttributeDepositBase = NftsAttributeDepositBase;
	type DepositPerByte = NftsDepositPerByte;

	type StringLimit = ConstU32<256>;
	type KeyLimit = ConstU32<64>;
	type ValueLimit = ConstU32<256>;
	type ApprovalsLimit = ConstU32<20>;
	type ItemAttributesApprovalsLimit = ConstU32<30>;
	type MaxTips = ConstU32<10>;
	type MaxDeadlineDuration = NftsMaxDeadlineDuration;
	type MaxAttributesPerCall = ConstU32<10>;
	type Features = NftsPalletFeatures;

	type OffchainSignature = Signature;
	type OffchainPublic = <Signature as Verify>::Signer;
	type WeightInfo = pallet_nfts::weights::SubstrateWeight<Runtime>;

	#[cfg(feature = "runtime-benchmarks")]
	type Helper = NftsBenchmarksHelper;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct NftsBenchmarksHelper;

#[cfg(feature = "runtime-benchmarks")]
impl pallet_nfts::BenchmarkHelper<CollectionId, MembershipId> for NftsBenchmarksHelper {
	fn collection(i: u16) -> CollectionId {
		i.into()
	}
	fn item(i: u16) -> MembershipId {
		MembershipId(
			<Runtime as pallet_communities::Config>::BenchmarkHelper::community_id(),
			i.into(),
		)
	}
}
