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
pub mod configuration;
pub mod constants;
pub mod contracts;
pub mod governance;
pub mod impls;
mod weights;
pub mod xcm_config;

pub use configuration::*;

use apis::*;
use cumulus_pallet_parachain_system::RelayNumberMonotonicallyIncreases;
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
	derive_impl,
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

use pallet_asset_tx_payment::ChargeAssetTxPayment;
use pallet_gas_transaction_payment::ChargeTransactionPayment as ChargeGasTxPayment;
use pallet_pass::ChargeTransactionToPassAccount as ChargeTxToPassAccount;
use pallet_skip_feeless_payment::SkipCheckIfFeeless;

// XCM Imports
use xcm::latest::prelude::BodyId;

pub use constants::{currency::*, fee::WeightToFee};

use pallet_payments::types::*;

pub use impls::{EqualOrGreatestRootCmp, ProxyType, RuntimeBlackListedCalls};

pub use parachains_common::{
	opaque, AccountId, AssetIdForTrustBackedAssets, AuraId, Balance, BlockNumber, Hash, Header, Nonce, Signature,
	AVERAGE_ON_INITIALIZE_RATIO, DAYS, HOURS, MAXIMUM_BLOCK_WEIGHT, MINUTES, NORMAL_DISPATCH_RATIO, SLOT_DURATION,
};

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, CommunityId>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

pub type ChargeTransaction = ChargeGasTxPayment<Runtime, ChargeAssetTxPayment<Runtime>>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	SkipCheckIfFeeless<Runtime, ChargeTxToPassAccount<ChargeTransaction, Runtime, ()>>,
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
	spec_version: 111,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 8,
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
#[frame_support::runtime]
mod runtime {
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeTask,
		RuntimeHoldReason,
		RuntimeFreezeReason
	)]
	pub struct Runtime;

	// System support stuff.
	#[runtime::pallet_index(0)]
	pub type System = frame_system;
	#[runtime::pallet_index(1)]
	pub type ParachainSystem = cumulus_pallet_parachain_system;
	#[runtime::pallet_index(2)]
	pub type Timestamp = pallet_timestamp;
	#[runtime::pallet_index(3)]
	pub type ParachainInfo = parachain_info;
	#[runtime::pallet_index(4)]
	pub type Origins = pallet_custom_origins;
	#[runtime::pallet_index(6)]
	pub type Pass = pallet_pass;

	// Monetary stuff.
	#[runtime::pallet_index(10)]
	pub type Balances = pallet_balances;
	#[runtime::pallet_index(11)]
	pub type TransactionPayment = pallet_transaction_payment;
	#[runtime::pallet_index(12)]
	pub type AssetsFreezer = pallet_assets_freezer<Instance1>;
	#[runtime::pallet_index(13)]
	pub type Assets = pallet_assets<Instance1>;
	#[runtime::pallet_index(14)]
	pub type AssetsTxPayment = pallet_asset_tx_payment;
	#[runtime::pallet_index(15)]
	pub type Vesting = pallet_vesting;
	#[runtime::pallet_index(16)]
	pub type SkipFeeless = pallet_skip_feeless_payment;
	#[runtime::pallet_index(17)]
	pub type GasTxPayment = pallet_gas_transaction_payment;

	// Collator support. The order of these 4 are important and shall not change.
	#[runtime::pallet_index(20)]
	pub type Authorship = pallet_authorship;
	#[runtime::pallet_index(21)]
	pub type CollatorSelection = pallet_collator_selection;
	#[runtime::pallet_index(22)]
	pub type Session = pallet_session;
	#[runtime::pallet_index(23)]
	pub type Aura = pallet_aura;
	#[runtime::pallet_index(24)]
	pub type AuraExt = cumulus_pallet_aura_ext;

	// XCM helpers.
	#[runtime::pallet_index(30)]
	pub type XcmpQueue = cumulus_pallet_xcmp_queue;
	#[runtime::pallet_index(31)]
	pub type PolkadotXcm = pallet_xcm;
	#[runtime::pallet_index(32)]
	pub type CumulusXcm = cumulus_pallet_xcm;
	#[runtime::pallet_index(33)]
	pub type MessageQueue = pallet_message_queue;

	// Utils
	#[runtime::pallet_index(42)]
	pub type Multisig = pallet_multisig;
	#[runtime::pallet_index(43)]
	pub type Utility = pallet_utility;
	#[runtime::pallet_index(44)]
	pub type Proxy = pallet_proxy;
	#[runtime::pallet_index(45)]
	pub type Scheduler = pallet_scheduler;
	#[runtime::pallet_index(46)]
	pub type Preimage = pallet_preimage;

	// Governance
	#[runtime::pallet_index(50)]
	pub type Treasury = pallet_treasury;
	#[runtime::pallet_index(51)]
	pub type KreivoCollective = pallet_ranked_collective<Instance1>;
	#[runtime::pallet_index(52)]
	pub type KreivoReferenda = pallet_referenda<Instance1>;

	// Virto Tooling
	#[runtime::pallet_index(60)]
	pub type Payments = pallet_payments;

	// Communities at Kreivo
	#[runtime::pallet_index(71)]
	pub type Communities = pallet_communities;
	#[runtime::pallet_index(72)]
	pub type CommunityTracks = pallet_referenda_tracks<Instance2>;
	#[runtime::pallet_index(73)]
	pub type CommunityReferenda = pallet_referenda<Instance2>;
	#[runtime::pallet_index(74)]
	pub type CommunityMemberships = pallet_nfts<Instance2>;
	#[runtime::pallet_index(75)]
	pub type CommunitiesManager = pallet_communities_manager;

	// Contracts
	#[runtime::pallet_index(80)]
	pub type Contracts = pallet_contracts;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type EventHandler = (CollatorSelection,);
}

parameter_types! {
	/// Relay Chain `TransactionByteFee` / 10
	pub const TransactionByteFee: Balance = 10 * MILLICENTS;
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

impl cumulus_pallet_aura_ext::Config for Runtime {}

/// How many parachain blocks are processed by the relay chain per parent.
/// Limits the number of blocks authored per slot.
const BLOCK_PROCESSING_VELOCITY: u32 = 1;
/// Maximum number of blocks simultaneously accepted by the Runtime, not yet
/// included into the relay chain.
const UNINCLUDED_SEGMENT_CAPACITY: u32 = 1;
/// Relay chain slot duration, in milliseconds.
const RELAY_CHAIN_SLOT_DURATION_MILLIS: u32 = 6_000;

/// Aura consensus hook
type ConsensusHook = cumulus_pallet_aura_ext::FixedVelocityConsensusHook<
	Runtime,
	RELAY_CHAIN_SLOT_DURATION_MILLIS,
	BLOCK_PROCESSING_VELOCITY,
	UNINCLUDED_SEGMENT_CAPACITY,
>;

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<100_000>;
	type AllowMultipleBlocksPerSlot = ConstBool<false>;
	type SlotDuration = ConstU64<SLOT_DURATION>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = ();
	// Enqueue XCMP messages from siblings for later processing.
	type XcmpQueue = TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>;
	type MaxActiveOutboundChannels = ConstU32<128>;
	// From https://github.com/polkadot-fellows/runtimes/blob/88da7d0abb9edbff027e57bc998a97c6ff10c18c/system-parachains/asset-hubs/asset-hub-kusama/src/lib.rs#L783:
	//
	// Most on-chain HRMP channels are configured to use 102400 bytes of max message
	// size, so we need to set the page size larger than that until we reduce the
	// channel size on-chain.
	type MaxPageSize = ConstU32<{ 103 * 1024 }>;
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

// === Parachain: Collators / Staking ====

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
	type Currency = Balances;
	type RejectOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type SpendPeriod = SpendPeriod;
	type Burn = ();
	type PalletId = TreasuryPalletId;
	type BurnDestination = ();
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
	type SpendFunds = ();
	type MaxApprovals = MaxApprovals;
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
		[pallet_contracts, Contracts]
		// XCM
		// NOTE: Make sure you point to the individual modules below.
		[pallet_xcm_benchmarks::fungible, XcmBalances]
		[pallet_xcm_benchmarks::generic, XcmGeneric]
	);
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
}
