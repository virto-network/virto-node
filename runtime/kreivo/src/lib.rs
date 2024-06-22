#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit.
#![recursion_limit = "256"]
#![allow(clippy::items_after_test_module)]

// // Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

#[cfg(test)]
mod tests;

pub mod apis;
pub mod constants;
pub mod governance;
pub mod impls;
mod weights;
pub mod xcm_config;

use apis::*;
use cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
use cumulus_primitives_core::{AggregateMessageOrigin, ParaId};
use governance::pallet_custom_origins;
use parachains_common::message_queue::{NarrowOriginToSibling, ParaIdToSibling};
use polkadot_runtime_common::xcm_sender::NoPriceForMessageDelivery;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::traits::{LookupError, StaticLookup, Verify};
pub use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, ConvertInto, IdentityLookup},
	transaction_validity::{InvalidTransaction, TransactionSource, TransactionValidity},
	AccountId32, ApplyExtrinsicResult, MultiAddress, Perbill, Percent, Permill,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use virto_common::CommunityId;

pub use apis::RuntimeApi;
pub use virto_common::FungibleAssetLocation;

use frame_support::{
	construct_runtime, derive_impl,
	dispatch::DispatchClass,
	ensure,
	genesis_builder_helper::{build_state, get_preset},
	parameter_types,
	traits::{
		fungible::HoldConsideration,
		fungibles,
		tokens::{imbalance::ResolveTo, PayFromAccount, UnityAssetBalanceConversion},
		AsEnsureOriginWithArg, ConstBool, ConstU32, ConstU64, ConstU8, Contains, EitherOf, EitherOfDiverse,
		EnsureOriginWithArg, LinearStoragePrice, NeverEnsureOrigin, TransformOrigin, WithdrawReasons,
	},
	weights::{constants::RocksDbWeight, ConstantMultiplier, Weight},
	BoundedVec, PalletId,
};

pub use frame_system::Call as SystemCall;

use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureRoot,
};

use pallet_nfts::PalletFeatures;

use pallet_xcm::{EnsureXcm, IsVoiceOfBody};
use xcm_config::{LocationConvertedConcreteId, RelayLocation, XcmOriginToTransactDispatchOrigin};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

// Polkadot imports
pub use polkadot_runtime_common::{prod_or_fast, BlockHashCount, SlowAdjustingFeeUpdate};

pub use weights::{BlockExecutionWeight, ExtrinsicBaseWeight};

// Kreivo Governance
pub mod collective;

// Virto toolchain
pub mod payments;

pub mod communities;

// XCM Imports
use xcm::latest::prelude::BodyId;

pub use constants::{currency::*, fee::WeightToFee};

use pallet_payments::types::*;

pub use impls::{EqualOrGreatestRootCmp, ProxyType, RuntimeBlackListedCalls};

pub use parachains_common::{
	opaque, AccountId, AssetIdForTrustBackedAssets, AuraId, Balance, BlockNumber, Hash, Header, Nonce, Signature,
	AVERAGE_ON_INITIALIZE_RATIO, DAYS, HOURS, MAXIMUM_BLOCK_WEIGHT, MINUTES, NORMAL_DISPATCH_RATIO, SLOT_DURATION,
};
pub use runtime_common::impls::AssetsToBlockAuthor;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, CommunityId>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_asset_tx_payment::ChargeAssetTxPayment<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;

/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive =
	frame_executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPalletsWithSystem>;

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("kreivo-parachain"),
	impl_name: create_runtime_str!("kreivo-parachain"),
	authoring_version: 1,
	spec_version: 110,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 7,
	state_version: 1,
};

/// The version information used to identify this runtime when compiled
/// natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

// Create the runtime by composing the FRAME pallets that were previously
// configured.
construct_runtime!(
	pub enum Runtime
	{
		// System support stuff.
		System: frame_system = 0,
		ParachainSystem: cumulus_pallet_parachain_system = 1,
		Timestamp: pallet_timestamp = 2,
		ParachainInfo: parachain_info = 3,
		Origins: pallet_custom_origins = 4,

		// Monetary stuff.
		Balances: pallet_balances = 10,
		TransactionPayment: pallet_transaction_payment = 11,
		Assets: pallet_assets::<Instance1> = 13,
		AssetTxPayment: pallet_asset_tx_payment::{Pallet, Storage, Event<T>} = 14,
		Vesting: pallet_vesting = 15,

		// Collator support. The order of these 4 are important and shall not change.
		Authorship: pallet_authorship = 20,
		CollatorSelection: pallet_collator_selection = 21,
		Session: pallet_session = 22,
		Aura: pallet_aura = 23,
		AuraExt: cumulus_pallet_aura_ext = 24,

		// XCM helpers.
		XcmpQueue: cumulus_pallet_xcmp_queue = 30,
		PolkadotXcm: pallet_xcm = 31,
		CumulusXcm: cumulus_pallet_xcm = 32,
		MessageQueue: pallet_message_queue = 33,

		// Utils
		Multisig: pallet_multisig = 42,
		Utility: pallet_utility = 43,
		Proxy: pallet_proxy = 44,
		Scheduler: pallet_scheduler = 45,
		Preimage: pallet_preimage = 46,

		// Governance
		Treasury: pallet_treasury = 50,
		KreivoCollective: pallet_ranked_collective::<Instance1> = 51,
		KreivoReferenda: pallet_referenda::<Instance1> = 52,

		// Virto Tooling
		Payments: pallet_payments = 60,

		// Communities at Kreivo
		Communities: pallet_communities = 71,
		CommunityTracks: pallet_referenda_tracks::<Instance2> = 72,
		CommunityReferenda: pallet_referenda::<Instance2> = 73,
		CommunityMemberships: pallet_nfts::<Instance2> = 74,
		CommunitiesManager: pallet_communities_manager = 75,
	}
);

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;

	// This part is copied from Substrate's `bin/node/runtime/src/lib.rs`.
	//  The `RuntimeBlockLength` and `RuntimeBlockWeights` exist here because the
	// `DeletionWeightLimit` and `DeletionQueueDepth` depend on those to parameterize
	// the lazy contract deletion.
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u16 = 2;
}

impl pallet_custom_origins::Config for Runtime {}

pub struct CommunityLookup;
impl StaticLookup for CommunityLookup {
	type Source = Address;
	type Target = AccountId;
	fn lookup(s: Self::Source) -> Result<Self::Target, LookupError> {
		match s {
			MultiAddress::Id(i) => Ok(i),
			MultiAddress::Index(i) => Ok(Communities::community_account(&i)),
			_ => Err(LookupError),
		}
	}
	fn unlookup(t: Self::Target) -> Self::Source {
		MultiAddress::Id(t)
	}
}

// Configure FRAME pallets to include in runtime.
#[derive_impl(frame_system::config_preludes::ParaChainDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	type Lookup = CommunityLookup;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	type Block = Block;
	type Nonce = Nonce;
	/// Maximum number of block number to block hash mappings to keep (oldest
	/// pruned first).
	type BlockHashCount = BlockHashCount;
	/// Runtime version.
	type Version = Version;
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// This is used as an identifier of the chain. 42 is the generic substrate
	/// prefix.
	type SS58Prefix = SS58Prefix;
	/// The action to take on a Runtime Upgrade
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	type WeightInfo = ();
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type EventHandler = (CollatorSelection,);
}

parameter_types! {
	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = ConstU32<50>;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type FreezeIdentifier = RuntimeFreezeReason;
	type MaxFreezes = ConstU32<256>;
	type RuntimeFreezeReason = RuntimeFreezeReason;
}

parameter_types! {
	/// Relay Chain `TransactionByteFee` / 10
	pub const TransactionByteFee: Balance = 10 * MILLICENTS;
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction =
		pallet_transaction_payment::FungibleAdapter<Balances, ResolveTo<TreasuryAccount, Balances>>;
	type WeightToFee = WeightToFee;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type OperationalFeeMultiplier = ConstU8<5>;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const RelayOrigin: AggregateMessageOrigin = AggregateMessageOrigin::Parent;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type OutboundXcmpMessageSource = XcmpQueue;
	type DmpQueue = frame_support::traits::EnqueueWithOrigin<MessageQueue, RelayOrigin>;
	type ReservedDmpWeight = ReservedDmpWeight;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
}

parameter_types! {
	pub MessageQueueServiceWeight: Weight = Perbill::from_percent(35) * RuntimeBlockWeights::get().max_block;
}

impl pallet_message_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type MessageProcessor =
		pallet_message_queue::mock_helpers::NoopMessageProcessor<cumulus_primitives_core::AggregateMessageOrigin>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MessageProcessor = xcm_builder::ProcessXcmMessage<
		AggregateMessageOrigin,
		xcm_executor::XcmExecutor<xcm_config::XcmConfig>,
		RuntimeCall,
	>;
	type Size = u32;
	// The XCMP queue pallet is only ever able to handle the `Sibling(ParaId)`
	// origin:
	type QueueChangeHandler = NarrowOriginToSibling<XcmpQueue>;
	type QueuePausedQuery = NarrowOriginToSibling<XcmpQueue>;
	type HeapSize = sp_core::ConstU32<{ 64 * 1024 }>;
	type MaxStale = sp_core::ConstU32<8>;
	type ServiceWeight = MessageQueueServiceWeight;
	type IdleMaxServiceWeight = ();
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

/// Relay chain slot duration, in milliseconds.
const RELAY_CHAIN_SLOT_DURATION_MILLIS: u32 = 6000;
/// How many parachain blocks are processed by the relay chain per parent.
/// Limits the number of blocks authored per slot.
const BLOCK_PROCESSING_VELOCITY: u32 = 1;
/// Maximum number of blocks simultaneously accepted by the Runtime, not yet
/// included into the relay chain.
const UNINCLUDED_SEGMENT_CAPACITY: u32 = 3;

/// Aura consensus hook
type ConsensusHook = cumulus_pallet_aura_ext::FixedVelocityConsensusHook<
	Runtime,
	RELAY_CHAIN_SLOT_DURATION_MILLIS,
	BLOCK_PROCESSING_VELOCITY,
	UNINCLUDED_SEGMENT_CAPACITY,
>;

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = ();
	// Enqueue XCMP messages from siblings for later processing.
	type XcmpQueue = TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>;
	type MaxInboundSuspended = sp_core::ConstU32<1_000>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = ();
	type PriceForSiblingDelivery = NoPriceForMessageDelivery<ParaId>;
}

parameter_types! {
	pub const Period: u32 = 6 * HOURS;
	pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionManager = CollatorSelection;
	// Essentially just Aura, but let's be pedantic.
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = ();
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<100_000>;
	type AllowMultipleBlocksPerSlot = ConstBool<false>;
	type SlotDuration = ConstU64<SLOT_DURATION>;
}

parameter_types! {
	pub const PotId: PalletId = PalletId(*b"PotStake");
	pub const MaxCandidates: u32 = 1000;
	pub const MinEligibleCollators: u32 = 5;
	pub const SessionLength: BlockNumber = 6 * HOURS;
	pub const MaxInvulnerables: u32 = 100;
	// StakingAdmin pluralistic body.
	pub const StakingAdminBodyId: BodyId = BodyId::Defense;
}

/// We allow root and the StakingAdmin to execute privileged collator selection
/// operations.
pub type CollatorSelectionUpdateOrigin =
	EitherOfDiverse<EnsureRoot<AccountId>, EnsureXcm<IsVoiceOfBody<RelayLocation, StakingAdminBodyId>>>;

impl pallet_collator_selection::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type UpdateOrigin = CollatorSelectionUpdateOrigin;
	type PotId = PotId;
	type MaxCandidates = MaxCandidates;
	type MinEligibleCollators = MinEligibleCollators;
	type MaxInvulnerables = MaxInvulnerables;
	// should be a multiple of session or things will get inconsistent
	type KickThreshold = Period;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = ();
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 2000 * CENTS;
	pub const ProposalBondMaximum: Balance = GRAND;
	pub const SpendPeriod: BlockNumber = 6 * DAYS;
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const TipCountdown: BlockNumber = DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 100 * CENTS;
	pub const DataDepositPerByte: Balance = CENTS;
	pub const MaxApprovals: u32 = 100;
	pub const MaxAuthorities: u32 = 100_000;
	pub const MaxKeys: u32 = 10_000;
	pub const MaxPeerInHeartbeats: u32 = 10_000;
	pub const MaxPeerDataEncodingSize: u32 = 1_000;
	pub TreasuryAccount: AccountId = Treasury::account_id();
	pub const PayoutSpendPeriod: BlockNumber = 30 * DAYS;

}

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type ApproveOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type RejectOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type OnSlash = Treasury;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type ProposalBondMaximum = ProposalBondMaximum;
	type SpendPeriod = SpendPeriod;
	type Burn = ();
	type BurnDestination = ();
	type MaxApprovals = MaxApprovals;
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
	type SpendFunds = ();
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<Balance>;
	type AssetKind = ();
	type Beneficiary = Self::AccountId;

	type BeneficiaryLookup = IdentityLookup<Self::Beneficiary>;
	type Paymaster = PayFromAccount<Balances, TreasuryAccount>;
	type BalanceConverter = UnityAssetBalanceConversion;
	type PayoutPeriod = PayoutSpendPeriod;
	#[cfg(feature = "runtime-benchmarks")]
	/// TODO: fix this benchmark helper in next release. We can proceed with the
	/// empty implementation. type BenchmarkHelper =
	/// polkadot_runtime_common::impls::benchmarks::TreasuryArguments;
	type BenchmarkHelper = ();
}

parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = deposit(0, 32);
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = ConstU32<100>;
	type WeightInfo = pallet_multisig::weights::SubstrateWeight<Runtime>;
}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const AssetDeposit: Balance = UNITS / 10; // 1 / 10 UNITS deposit to create asset
	pub const AssetAccountDeposit: Balance = deposit(1, 16);
	pub const ApprovalDeposit: Balance = EXISTENTIAL_DEPOSIT;
	pub const AssetsStringLimit: u32 = 50;
	/// Key = 32 bytes, Value = 36 bytes (32+1+1+1+1)
	// https://github.com/paritytech/substrate/blob/069917b/frame/assets/src/lib.rs#L257L271
	pub const MetadataDepositBase: Balance = deposit(1, 68);
	pub const MetadataDepositPerByte: Balance = deposit(0, 1);
}

/// We allow root to execute privileged asset operations.

pub type AssetsForceOrigin = EnsureRoot<AccountId>;
pub type KreivoAssetsInstance = pallet_assets::Instance1;
type KreivoAssetsCall = pallet_assets::Call<Runtime, KreivoAssetsInstance>;

impl pallet_assets::Config<KreivoAssetsInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = FungibleAssetLocation;
	type AssetIdParameter = FungibleAssetLocation;
	type Currency = Balances;
	/// Only root can create assets and force state changes.
	type CreateOrigin = AsEnsureOriginWithArg<NeverEnsureOrigin<AccountId>>;
	type ForceOrigin = AssetsForceOrigin;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = AssetsStringLimit;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = weights::pallet_assets::WeightInfo<Runtime>;
	type CallbackHandle = ();
	type AssetAccountDeposit = AssetAccountDeposit;
	type RemoveItemsLimit = frame_support::traits::ConstU32<1000>;
	type MaxHolds = frame_support::traits::ConstU32<50>;
	type RuntimeHoldReason = RuntimeHoldReason;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = deposit(1, 40);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = deposit(0, 33);
	pub const MaxProxies: u16 = 32;
	// One storage item; key size 32, value size 16
	pub const AnnouncementDepositBase: Balance = deposit(1, 48);
	pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
	pub const MaxPending: u16 = 32;
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type WeightInfo = weights::pallet_proxy::WeightInfo<Runtime>;
	type MaxPending = MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * RuntimeBlockWeights::get().max_block;
}

#[cfg(not(feature = "runtime-benchmarks"))]
parameter_types! {
	pub const MaxScheduledPerBlock: u32 = 50;
}

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub const MaxScheduledPerBlock: u32 = 200;
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
	type OriginPrivilegeCmp = EqualOrGreatestRootCmp;
	type Preimages = Preimage;
}

parameter_types! {
	pub const PreimageBaseDeposit: Balance = deposit(2, 64);
	pub const PreimageByteDeposit: Balance = deposit(0, 1);
	pub const PreimageHoldReason: RuntimeHoldReason = RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
}

impl pallet_preimage::Config for Runtime {
	type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type Consideration = HoldConsideration<
		AccountId,
		Balances,
		PreimageHoldReason,
		LinearStoragePrice<PreimageBaseDeposit, PreimageByteDeposit, Balance>,
	>;
}

parameter_types! {
	pub NftsPalletFeatures: PalletFeatures = PalletFeatures::all_enabled();
	pub const NftsMaxDeadlineDuration: BlockNumber = 12 * 30 * DAYS;
	// From https://github.com/polkadot-fellows/runtimes/blob/main/system-parachains/asset-hubs/asset-hub-kusama/src/lib.rs#L745
	pub const NftsCollectionDeposit: Balance = UNITS / 10;
	pub const NftsItemDeposit: Balance = UNITS / 1_000;
	pub const NftsMetadataDepositBase: Balance = MetadataDepositBase::get();
	pub const NftsAttributeDepositBase: Balance = deposit(1, 0);
	pub const NftsDepositPerByte: Balance = MetadataDepositPerByte::get();
}

parameter_types! {
	/// The asset ID for the asset that we use to pay for message delivery fees.
	pub FeeAssetId: cumulus_primitives_core::AssetId = xcm_config::RelayLocation::get().into();
	/// The base fee for the message delivery fees.
	pub const ToSiblingBaseDeliveryFee: u128 = CENTS.saturating_mul(3);
	pub const ToParentBaseDeliveryFee: u128 = CENTS.saturating_mul(3);
}

pub type PriceForSiblingParachainDelivery = polkadot_runtime_common::xcm_sender::ExponentialPrice<
	FeeAssetId,
	ToSiblingBaseDeliveryFee,
	TransactionByteFee,
	XcmpQueue,
>;
pub type PriceForParentDelivery = polkadot_runtime_common::xcm_sender::ExponentialPrice<
	FeeAssetId,
	ToParentBaseDeliveryFee,
	TransactionByteFee,
	ParachainSystem,
>;

impl pallet_asset_tx_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Fungibles = Assets;
	type OnChargeAssetTransaction = pallet_asset_tx_payment::FungiblesAdapter<
		pallet_assets::BalanceToAssetBalance<Balances, Runtime, ConvertInto, KreivoAssetsInstance>,
		AssetsToBlockAuthor<Runtime, KreivoAssetsInstance>,
	>;
}

parameter_types! {
	pub const MinVestedTransfer: Balance = 100 * CENTS;
	pub UnvestedFundsAllowedWithdrawReasons: WithdrawReasons =
		WithdrawReasons::except(WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE);
}

impl pallet_vesting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BlockNumberToBalance = ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = pallet_vesting::weights::SubstrateWeight<Runtime>;
	type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
	type BlockNumberProvider = System;
	const MAX_VESTING_SCHEDULES: u32 = 28;
}

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	frame_benchmarking::define_benchmarks!(
		[frame_system, SystemBench::<Runtime>]
		[pallet_balances, Balances]
		[pallet_session, SessionBench::<Runtime>]
		[pallet_timestamp, Timestamp]
		[pallet_collator_selection, CollatorSelection]
		[cumulus_pallet_xcmp_queue, XcmpQueue]
		[pallet_treasury, Treasury]
		[pallet_multisig, Multisig]
		[pallet_vesting, Vesting]
		[pallet_utility, Utility]
		[pallet_assets, Assets]
		[pallet_proxy, Proxy]
		[pallet_referenda, KreivoReferenda]
		[pallet_ranked_collective, KreivoCollective]
		[pallet_payments, Payments]
		[pallet_communities, Communities]
		[pallet_referenda_tracks, CommunityTracks]
		[pallet_referenda, CommunityReferenda]
		[pallet_communities_manager, CommunitiesManager]
		[pallet_nfts, CommunityMemberships]
		// XCM
		// NOTE: Make sure you point to the individual modules below.
		[pallet_xcm_benchmarks::fungible, XcmBalances]
		[pallet_xcm_benchmarks::generic, XcmGeneric]
	);
}

struct CheckInherents;
impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data = cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
			relay_chain_slot,
			sp_std::time::Duration::from_secs(6),
		)
		.create_inherent_data()
		.expect("Could not create the timestamp inherent data");

		inherent_data.check_extrinsics(block)
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
}
