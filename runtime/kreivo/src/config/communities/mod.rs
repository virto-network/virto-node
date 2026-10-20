use super::*;

use frame_support::traits::TryMapSuccess;
use frame_system::{EnsureRootWithSuccess, EnsureSigned};
use pallet_communities::origin::{EnsureCommunity, EnsureSignedPays};
use sp_runtime::{morph_types, traits::AccountIdConversion};
use virto_common::{CommunityId, MembershipId};

pub use memberships::{Memberships, MembershipsManagerAccount, RetiringMembershipsAccount};

pub mod governance;
pub mod memberships;

type CreationPayment = Option<(Balance, AccountId, AccountId)>;

parameter_types! {
	pub const CommunityPalletId: PalletId = PalletId(*b"kv/cmtys");
	pub const MembershipsCollectionId: CommunityId = 0;
	pub const MembershipNftAttr: &'static [u8; 10] = b"membership";
	pub const CommunityDepositAmount: Balance = UNITS / 2;
	pub const NoPay: CreationPayment = None;
	/// What each issued membership holds on its community's account (`DEC-19`): the existential
	/// deposit, provisional until the economic study (0007-A1; `deployment/kreivo-0.18.md` §1).
	pub const MembershipDepositAmount: Balance = EXISTENTIAL_DEPOSIT;
	/// How long a retiring membership waits: 10 days of relay chain blocks (m3-retirement-delay).
	/// Both change only by runtime upgrade.
	pub const RetirementDelayInBlocks: BlockNumber = 10 * DAYS;
	/// The most memberships a call issues or retires.
	pub const MaxIssuePerCall: u32 = 1_024;
	/// A TEST VALUE: the part of a registration that the community's account holds. `F-07b` sets the
	/// measured one (0010-Q2, `DEP-21`).
	pub const CommunityStoragePartAmount: Balance = UNITS / 10;
	/// The rest of the registration payment, paid to the Treasury: 0.5 KSM less the storage part
	/// (0010-Q1: two amounts whose sum is the payment).
	pub const CommunityConsumptionPartAmount: Balance = UNITS / 2 - UNITS / 10;
}

morph_types! {
	pub type AccountToCommunityId: TryMorph = |a: AccountId| -> Result<CommunityId, ()> {
		PalletId::try_from_sub_account(&a).map(|(_, id)| id).ok_or(())
	};
}
pub(crate) type EnsureCommunityAccount = TryMapSuccess<EnsureSigned<AccountId>, AccountToCommunityId>;

type RootCreatesCommunitiesForFree = EnsureRootWithSuccess<AccountId, NoPay>;
type AnyoneElsePays = EnsureSignedPays<Runtime, CommunityDepositAmount, TreasuryAccount>;

impl pallet_communities::Config for Runtime {
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type WeightInfo = weights::pallet_communities::WeightInfo<Runtime>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type CreateOrigin = frame_system::EnsureNever<CreationPayment>;

	#[cfg(feature = "runtime-benchmarks")]
	type CreateOrigin = RootCreatesCommunitiesForFree;

	type AdminOrigin = EitherOf<EnsureCommunity<Self>, EnsureCommunityAccount>;
	type MemberMgmtOrigin = EitherOf<EnsureCommunity<Self>, EnsureCommunityAccount>;
	// A member acts on a membership they hold (`DEC-31`).
	type MemberOrigin = EnsureSigned<AccountId>;
	type CommunityId = CommunityId;

	type MembershipId = MembershipId;
	type MemberMgmt = Memberships;
	type Polls = CommunityReferenda;
	type Assets = Assets;

	type AssetsFreezer = AssetsFreezer;

	type Balances = Balances;
	type BlockNumberProvider = RelaychainData;

	type PalletId = CommunityPalletId;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = CommunityBenchmarkHelper;
}

impl pallet_communities_manager::Config for Runtime {
	type CreateCollection = CommunityMemberships;
	type Tracks = CommunityTracks;
	type RankedCollective = KreivoCollective;
	type WeightInfo = weights::pallet_communities_manager::WeightInfo<Self>;
	type RegisterOrigin = EitherOf<RootCreatesCommunitiesForFree, AnyoneElsePays>;
	type RuntimeHoldReason = RuntimeHoldReason;
	type GroupAdminOrigin = EitherOf<EnsureCommunity<Runtime>, EnsureCommunityAccount>;
	type Issuer = Memberships;
	type ItemAttributes =
		pallet_communities_manager::ClearNftsAttributes<Runtime, memberships::CommunityMembershipsInstance>;
	type MembershipsManagerAccount = MembershipsManagerAccount;
	type CommunityStoragePart = CommunityStoragePartAmount;
	type CommunityConsumptionPart = CommunityConsumptionPartAmount;
	type RetirementHolder = RetiringMembershipsAccount;
	type MembershipDeposit = MembershipDepositAmount;
	type RetirementDelay = RetirementDelayInBlocks;
	type MaxIssuePerCall = MaxIssuePerCall;
}

#[cfg(feature = "runtime-benchmarks")]
pub use benchmarks::CommunityBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks {
	use super::*;

	use alloc::boxed::Box;
	use governance::{CommunityReferendaInstance, CommunityTracksInstance};

	use frame_benchmarking::BenchmarkError;
	use frame_contrib_traits::memberships::Issue;
	use frame_support::traits::{nonfungibles_v2::Create, schedule::DispatchTime};
	use frame_system::pallet_prelude::{OriginFor, RuntimeCallFor};
	use pallet_communities::{
		types::{CommunityIdOf, MembershipIdOf, PalletsOriginOf, PollIndexOf},
		BenchmarkHelper,
	};
	use pallet_referenda::{BoundedCallOf, Curve, Pallet as Referenda, TrackInfo};
	use pallet_referenda_tracks::Pallet as Tracks;
	use parity_scale_codec::Encode;
	use sp_runtime::Perbill;

	/// Moves time to block `n`. Referenda (and the scheduler) count relay chain blocks, so the
	/// relay chain block number moves along with the parachain's.
	fn advance_to(n: BlockNumber) {
		System::set_block_number(n);
		<RelaychainData as sp_runtime::traits::BlockNumberProvider>::set_block_number(n);
	}

	pub struct CommunityBenchmarkHelper;

	impl BenchmarkHelper<Runtime> for CommunityBenchmarkHelper {
		fn community_id() -> CommunityIdOf<Runtime> {
			1
		}
		fn community_asset_id() -> AssetIdOf<Runtime> {
			1u32.into()
		}
		fn community_desired_size() -> u32 {
			u8::MAX.into()
		}

		fn initialize_memberships_collection() -> Result<(), BenchmarkError> {
			let community_id = Self::community_id();
			let community_account = pallet_communities::Pallet::<Runtime>::community_account(&community_id);

			CommunityMemberships::create_collection_with_id(
				MembershipsCollectionId::get(),
				&community_account,
				&community_account,
				&Default::default(),
			)?;

			// A community's collection is owned and administered by the manager account (`DEC-36`).
			let manager = MembershipsManagerAccount::get();
			CommunityMemberships::create_collection_with_id(
				Self::community_id(),
				&manager,
				&manager,
				&Default::default(),
			)?;

			Ok(())
		}

		fn issue_membership(
			community_id: CommunityIdOf<Runtime>,
			membership_id: MembershipIdOf<Runtime>,
		) -> Result<(), BenchmarkError> {
			<Memberships as Issue<AccountId>>::issue(&community_id, &membership_id)?;

			Ok(())
		}

		fn prepare_track(pallet_origin: PalletsOriginOf<Runtime>) -> Result<(), BenchmarkError> {
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

			Tracks::<Runtime, CommunityTracksInstance>::insert(RuntimeOrigin::root(), id, info, pallet_origin)?;

			Ok(())
		}

		fn prepare_poll(
			origin: OriginFor<Runtime>,
			proposal_origin: PalletsOriginOf<Runtime>,
			proposal_call: RuntimeCallFor<Runtime>,
		) -> Result<PollIndexOf<Runtime>, BenchmarkError> {
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

			advance_to(2);
			Referenda::<Runtime, CommunityReferendaInstance>::nudge_referendum(RuntimeOrigin::root(), 0)?;

			Ok(0)
		}

		fn finish_poll(index: PollIndexOf<Runtime>) -> Result<(), BenchmarkError> {
			advance_to(8);
			Referenda::<Runtime, CommunityReferendaInstance>::nudge_referendum(RuntimeOrigin::root(), index)?;

			frame_support::assert_ok!(Referenda::<Runtime, CommunityReferendaInstance>::ensure_ongoing(index));

			advance_to(9);
			Referenda::<Runtime, CommunityReferendaInstance>::nudge_referendum(RuntimeOrigin::root(), index)?;

			frame_support::assert_err!(
				Referenda::<Runtime, CommunityReferendaInstance>::ensure_ongoing(index),
				pallet_referenda::Error::<Runtime, CommunityReferendaInstance>::NotOngoing
			);

			Ok(())
		}
	}
}
