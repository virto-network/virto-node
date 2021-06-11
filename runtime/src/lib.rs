#![allow(
    clippy::large_enum_variant,
    clippy::from_over_into,
    missing_debug_implementations
)]
#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use frame_system::EnsureRoot;
use sp_api::impl_runtime_apis;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, IdentifyAccount, Verify};
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, FixedU128, MultiSignature,
};

#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

mod proxy_type;
use proxy_type::ProxyType;
use sp_std::prelude::*;
use vln_primitives::{Asset, DefaultRateCombinator};
mod mocks;
use mocks::MockAssets;

#[cfg(feature = "standalone")]
use standalone_use::*;
#[cfg(feature = "standalone")]
mod standalone_use {
    pub use pallet_grandpa::{
        fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
    };
    pub use sp_core::sr25519;
    pub use sp_runtime::traits::NumberFor;
}

#[cfg(not(feature = "standalone"))]
mod currency_id_convert;

// XCM imports
// #[cfg(not(feature = "standalone"))]
// use parachain_use::*;
// #[cfg(not(feature = "standalone"))]
// mod parachain_use {
//     //pub use crate::currency_id_convert::CurrencyIdConvert;
//     pub use frame_system::EnsureRoot;
//     // pub use orml_xcm_support::{
//     //     IsNativeConcrete, MultiCurrencyAdapter, MultiNativeAsset, XcmHandler as XcmHandlerT,
//     // };
//     pub use polkadot_parachain::primitives::Sibling;
//     pub use sp_runtime::{
//         traits::{Convert, Identity},
//         DispatchResult,
//     };
//     pub use sp_std::collections::btree_set::BTreeSet;
//     pub use vln_primitives::NetworkAsset;
//     // pub use xcm::v0::{
//     //     Junction::{Parachain, Parent},
//     //     MultiLocation::{self, X1, X2},
//     //     NetworkId, Xcm,
//     // };
//     // pub use xcm_builder::{
//     //     AccountId32Aliases, AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom,
//     //     EnsureXcmOrigin, FixedRateOfConcreteFungible, FixedWeightBounds, IsConcrete,
//     //     LocationInverter, NativeAsset, ParentIsDefault, RelayChainAsNative,
//     //     SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
//     //     SovereignSignedViaLocation, TakeWeightCredit,
//     // };
//     //pub use xcm_executor::{Config, XcmExecutor};
// }

use frame_system::limits::{BlockLength, BlockWeights};

// A few exports that help ease life for downstream crates.
pub use frame_support::{
    construct_runtime, parameter_types,
    traits::{Get, KeyOwnerProofSystem, Randomness},
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        DispatchClass, IdentityFee, Weight,
    },
    StorageValue,
};
pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Signed version of Balance.
pub type Amount = i64;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    pub type BlockId = generic::BlockId<Block>;
}

#[cfg(not(feature = "standalone"))]
impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: Aura,
    }
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("VLN-PC"),
    impl_name: create_runtime_str!("vln-runtime"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
};

#[cfg(feature = "standalone")]
pub const MILLISECS_PER_BLOCK: u64 = 3000;

#[cfg(not(feature = "standalone"))]
pub const MILLISECS_PER_BLOCK: u64 = 6000; // ensure to align with relay chain - 6sec for rococo/ksm/polkadot

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

pub const UNITS: Balance = 1_000_000_000_000;
pub const CENTS: Balance = UNITS / 30_000;
pub const GRAND: Balance = CENTS * 100_000;
pub const MILLICENTS: Balance = CENTS / 1_000;

// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

/// We assume that ~10% of the block weight is consumed by `on_initalize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

parameter_types! {
    pub const BlockHashCount: BlockNumber = 250;
    pub const Version: RuntimeVersion = VERSION;
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
    pub const SS58Prefix: u8 = 42;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = ();
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = RuntimeBlockWeights;
    /// The maximum length of a block (in bytes).
    type BlockLength = RuntimeBlockLength;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = AccountIdLookup<AccountId, ()>;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// Version of the runtime.
    type Version = Version;
    /// Converts a module to the index of the module in `construct_runtime!`.
    ///
    /// This type is being generated by `construct_runtime!`.
    type PalletInfo = PalletInfo;
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = ();
    /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = SS58Prefix;
    #[cfg(feature = "standalone")]
    type OnSetCode = ();
    #[cfg(not(feature = "standalone"))]
    type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

impl pallet_sudo::Config for Runtime {
    type Event = Event;
    type Call = Call;
}

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
}

parameter_types! {
    pub const ProxyDepositBase: Balance = 1;
    pub const ProxyDepositFactor: Balance = 1;
    pub const MaxProxies: u16 = 4;
    pub const MaxPending: u32 = 2;
    pub const AnnouncementDepositBase: Balance = 1;
    pub const AnnouncementDepositFactor: Balance = 1;
}

impl pallet_proxy::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type Currency = Balances;
    type ProxyType = ProxyType;
    type ProxyDepositBase = ProxyDepositBase;
    type ProxyDepositFactor = ProxyDepositFactor;
    type MaxProxies = MaxProxies;
    type WeightInfo = ();
    type CallHasher = BlakeTwo256;
    type MaxPending = MaxPending;
    type AnnouncementDepositBase = AnnouncementDepositBase;
    type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

parameter_types! {
    pub const MinimumCount: u32 = 3;
    pub const ExpiresIn: u32 = 600;
    pub RootOperatorAccountId: AccountId = Sudo::key();
}

impl orml_oracle::Config for Runtime {
    type Event = Event;
    type OnNewData = ();
    type CombineData = orml_oracle::DefaultCombineData<Runtime, MinimumCount, ExpiresIn>;
    type Time = Timestamp;
    type OracleKey = Asset;
    type OracleValue = FixedU128;
    type RootOperatorAccountId = RootOperatorAccountId;
    type WeightInfo = ();
    type Members = Whitelist;
}

parameter_types! {
    pub const MaxMembers : u32 = 100;
}

impl pallet_membership::Config for Runtime {
    type Event = Event;
    type AddOrigin = EnsureRoot<AccountId>;
    type RemoveOrigin = EnsureRoot<AccountId>;
    type SwapOrigin = EnsureRoot<AccountId>;
    type ResetOrigin = EnsureRoot<AccountId>;
    type PrimeOrigin = EnsureRoot<AccountId>;
    type MaxMembers = MaxMembers;
    type WeightInfo = ();
    type MembershipInitialized = ();
    type MembershipChanged = ();
}

impl vln_rate_provider::Config for Runtime {
    type Event = Event;
    type Asset = Asset;
    type BaseAsset = Asset;
    type Whitelist = Whitelist;
    type PriceFeed = Oracle;
    type OracleValue = FixedU128;
    type RateCombinator = DefaultRateCombinator;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 0;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = MaxLocks;
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const TransactionByteFee: u128 = MILLICENTS;
}

impl pallet_transaction_payment::Config for Runtime {
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ();
}

parameter_types! {
    pub const AssetDepositGeneral: Balance = UNITS; // 1 UNIT deposit to create asset
    pub const ApprovalDepositGeneral: Balance = UNITS;
    pub const StringLimitGeneral: u32 = 50;
    pub const MetadataDepositBaseGeneral: Balance = UNITS;
    pub const MetadataDepositPerByteGeneral: Balance = UNITS;
}

type GeneralAssets = pallet_assets::Instance1;
impl pallet_assets::Config<GeneralAssets> for Runtime {
    type Event = Event;
    type Balance = Balance;
    type AssetId = u32;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>; // allow council later
    type AssetDeposit = AssetDepositGeneral;
    type MetadataDepositBase = MetadataDepositBaseGeneral;
    type MetadataDepositPerByte = MetadataDepositPerByteGeneral;
    type ApprovalDeposit = ApprovalDepositGeneral;
    type StringLimit = StringLimitGeneral;
    type Freezer = ();
    type Extra = ();
    type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const AssetDepositFiat: Balance = UNITS; // 1 UNIT deposit to create asset
    pub const ApprovalDepositFiat: Balance = UNITS;
    pub const StringLimitFiat: u32 = 50;
    pub const MetadataDepositBaseFiat: Balance = UNITS;
    pub const MetadataDepositPerByteFiat: Balance = UNITS;
}

type FiatAssets = pallet_assets::Instance2;
impl pallet_assets::Config<FiatAssets> for Runtime {
    type Event = Event;
    type Balance = Balance;
    type AssetId = u32;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>; // allow council later
    type AssetDeposit = AssetDepositFiat;
    type MetadataDepositBase = MetadataDepositBaseFiat;
    type MetadataDepositPerByte = MetadataDepositPerByteFiat;
    type ApprovalDeposit = ApprovalDepositFiat;
    type StringLimit = StringLimitFiat;
    type Freezer = ();
    type Extra = ();
    type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
}

impl vln_escrow::Config for Runtime {
    type Event = Event;
    type Asset = MockAssets;
    type JudgeWhitelist = Whitelist;
}

parameter_types! {
    pub const ClassDeposit: u64 = 2;
    pub const InstanceDeposit: u64 = 1;
    pub const KeyLimit: u32 = 50;
    pub const ValueLimit: u32 = 50;
    pub const StringLimit: u32 = 50;
    pub const MetadataDepositBase: u64 = 1;
    pub const AttributeDepositBase: u64 = 1;
    pub const MetadataDepositPerByte: u64 = 1;
}

impl pallet_uniques::Config for Runtime {
    type Event = Event;
    type ClassId = u32;
    type InstanceId = u32;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>; // allow council later
    type ClassDeposit = ClassDeposit;
    type InstanceDeposit = InstanceDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type AttributeDepositBase = AttributeDepositBase;
    type DepositPerByte = MetadataDepositPerByte;
    type StringLimit = StringLimit;
    type KeyLimit = KeyLimit;
    type ValueLimit = ValueLimit;
    type WeightInfo = ();
}

#[cfg(feature = "standalone")]
pub use standalone_impl::*;

#[cfg(feature = "standalone")]
mod standalone_impl {
    use super::*;

    impl_opaque_keys! {
        pub struct SessionKeys {
            pub aura: Aura,
            pub grandpa: Grandpa,
        }
    }

    impl pallet_grandpa::Config for Runtime {
        type Call = Call;
        type Event = Event;
        type KeyOwnerProofSystem = ();
        type KeyOwnerProof =
            <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
        type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
            KeyTypeId,
            GrandpaId,
        )>>::IdentificationTuple;
        type HandleEquivocation = ();
        type WeightInfo = ();
    }
}

#[cfg(not(feature = "standalone"))]
pub use parachain_impl::*;
#[cfg(not(feature = "standalone"))]
mod parachain_impl {
    use super::*;

    parameter_types! {
        pub ReservedDmpWeight: Weight = RuntimeBlockWeights::get().max_block / 4;
    }

    impl cumulus_pallet_parachain_system::Config for Runtime {
        type Event = Event;
        type OnValidationData = ();
        type SelfParaId = ParachainInfo;
        type DmpMessageHandler = ();
        type ReservedDmpWeight = ReservedDmpWeight;
        type OutboundXcmpMessageSource = ();
        type XcmpMessageHandler = ();
        type ReservedXcmpWeight = ();
    }

    // impl orml_unknown_tokens::Config for Runtime {
    //     type Event = Event;
    // }

    impl parachain_info::Config for Runtime {}

    impl cumulus_pallet_aura_ext::Config for Runtime {}

    // /// The means for routing XCM messages which are not for local execution into the right message
    // /// queues.
    // pub type XcmRouter = (
    //     // Two routers - use UMP to communicate with the relay chain:
    //     cumulus_primitives_utility::ParentAsUmp<ParachainSystem>,
    //     // ..and XCMP to communicate with the sibling chains.
    //     XcmpQueue,
    // );

    // impl pallet_xcm::Config for Runtime {
    //     type Event = Event;
    //     type SendXcmOrigin = EnsureXcmOrigin<Origin, ()>; // No allowed origins
    //     type XcmRouter = XcmRouter;
    //     type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, ()>; // No allowed origins
    //     type XcmExecutor = XcmExecutor<XcmConfig>;
    // }

    // impl cumulus_pallet_xcm::Config for Runtime {}

    // impl cumulus_pallet_xcmp_queue::Config for Runtime {
    //     type Event = Event;
    //     type XcmExecutor = XcmExecutor<XcmConfig>;
    //     type ChannelInfo = ParachainSystem;
    // }

    // parameter_types! {
    //     pub const RococoLocation: MultiLocation = X1(Parent);
    //     pub VlnNetwork: NetworkId = NetworkId::Named("vln".into());
    //     pub const PolkadotNetwork: NetworkId = NetworkId::Polkadot;
    //     pub const GetUsdvId: Asset = Asset::Usdv;
    //     pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
    //     pub Ancestry: MultiLocation = Parachain {
    //         id: ParachainInfo::parachain_id().into()
    //     }.into();
    //     pub const RelayChainCurrencyId: NetworkAsset = NetworkAsset::DOT;
    // }

    // type LocationConverter = (
    //     ParentIsDefault<AccountId>,
    //     SiblingParachainConvertsVia<Sibling, AccountId>,
    //     AccountId32Aliases<VlnNetwork, AccountId>,
    // );

    // type LocalOriginConverter = (
    //     SovereignSignedViaLocation<LocationConverter, Origin>,
    //     RelayChainAsNative<RelayChainOrigin, Origin>,
    //     SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
    //     SignedAccountId32AsNative<VlnNetwork, Origin>,
    // );

    // type LocalAssetTransactor = xcm_builder::CurrencyAdapter<
    //     // Use this currency:
    //     Tokens,
    //     // Use this currency when it is a fungible asset matching the given location or name:
    //     // TODO: This impl handles the case of relay<->parachain, but DOT cannot be stored on
    //     // Centrifuge so this is not useful. We can just re-implement the type w/o relay case.
    //     IsConcrete<RococoLocation>,
    //     // Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
    //     LocationConverter,
    //     // Our chain's account ID type (we can't get away without mentioning it explicitly):
    //     AccountId,
    // >;

    // pub type Barrier = (
    //     TakeWeightCredit,
    //     AllowTopLevelPaidExecutionFrom<All<MultiLocation>>,
    //     AllowUnpaidExecutionFrom<All<MultiLocation>>,
    // );

    // pub struct XcmConfig;
    // impl Config for XcmConfig {
    //     type Call = Call;
    //     type XcmSender = XcmRouter;
    //     // How to withdraw and deposit an asset.
    //     type AssetTransactor = LocalAssetTransactor;
    //     type OriginConverter = LocalOriginConverter;
    //     type IsReserve = NativeAsset;
    //     type IsTeleporter = (); // <- should be enough to allow teleportation of ROC
    //     type LocationInverter = LocationInverter<Ancestry>;
    //     type Barrier = Barrier;
    //     type Weigher = FixedWeightBounds<UnitWeightCost, Call>;
    //     type Trader = FixedRateOfConcreteFungible<WeightPrice>;
    //     type ResponseHandler = (); // Don't handle responses for now.
    // }

    // impl cumulus_pallet_xcm_handler::Config for Runtime {
    //     type Event = Event;
    //     type XcmExecutor = XcmExecutor<XcmConfig>;
    //     type UpwardMessageSender = ParachainSystem;
    //     type XcmpMessageSender = ParachainSystem;
    //     type SendXcmOrigin = EnsureRoot<AccountId>;
    //     type AccountIdConverter = LocationConverter;
    // }

    // parameter_types! {
    //     pub const GetRelayChainId: NetworkId = NetworkId::Polkadot;
    // }

    // pub struct AccountId32Convert;
    // impl Convert<AccountId, [u8; 32]> for AccountId32Convert {
    //     fn convert(account_id: AccountId) -> [u8; 32] {
    //         account_id.into()
    //     }
    // }

    // pub struct HandleXcm;
    // impl XcmHandlerT<AccountId> for HandleXcm {
    //     fn execute_xcm(origin: AccountId, xcm: Xcm) -> DispatchResult {
    //         XcmHandler::execute_xcm(origin, xcm)
    //     }
    // }

    // parameter_types! {
    //     pub SelfLocation: MultiLocation = X2(Parent, Parachain { id: ParachainInfo::get().into() });
    // }

    // impl orml_xtokens::Config for Runtime {
    //     type Event = Event;
    //     type Balance = Balance;
    //     type CurrencyId = NetworkAsset;
    //     type AccountId32Convert = AccountId32Convert;
    //     type CurrencyIdConvert = CurrencyIdConvert;
    //     type SelfLocation = SelfLocation;
    //     type XcmHandler = HandleXcm;
    // }
}

macro_rules! construct_vln_runtime {
	($( $modules:tt )*) => {
            // Create the runtime by composing the FRAME pallets that were previously configured.
            construct_runtime!{
                pub enum Runtime where
                    Block = Block,
                    NodeBlock = opaque::Block,
                    UncheckedExtrinsic = UncheckedExtrinsic,
                {
                    System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
                    Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
                    RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Call, Storage},
                    Sudo: pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>},
                    TransactionPayment: pallet_transaction_payment::{Pallet, Storage},
                    Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
                    Aura: pallet_aura::{Config<T>, Pallet},
                    Assets: pallet_assets::<Instance1>::{Pallet, Call, Storage, Event<T>},
                    Fiat: pallet_assets::<Instance2>::{Pallet, Call, Storage, Event<T>},

                    // vln dependencies
                    Whitelist: pallet_membership::{Call, Storage, Pallet, Event<T>, Config<T>},
                    Proxy: pallet_proxy::{Call, Event<T>, Pallet, Storage},
                    Oracle: orml_oracle::{Call, Event<T>, Pallet, Storage},
                    RatesProvider: vln_rate_provider::{Call, Event<T>, Pallet, Storage},
                    Escrow: vln_escrow::{Call, Event<T>, Pallet, Storage},
                    Uniques: pallet_uniques::{Pallet, Call, Storage, Event<T>},
                    $($modules)*
                }
            }
    }
}

#[cfg(feature = "standalone")]
construct_vln_runtime! {
    Grandpa: pallet_grandpa::{Call, Config, Event, Pallet, Storage},
}

#[cfg(not(feature = "standalone"))]
construct_vln_runtime! {
    AuraExt: cumulus_pallet_aura_ext::{Pallet, Config},
    ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event<T>},
    ParachainInfo: parachain_info::{Pallet, Storage, Config},
    // XTokens: orml_xtokens::{Pallet, Storage, Call, Event<T>},
    // UnknownTokens: orml_unknown_tokens::{Pallet, Storage, Event},
    // PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin},
    // CumulusXcm: cumulus_pallet_xcm::{Pallet, Origin},
    // XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>},
}

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPallets,
>;

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            Runtime::metadata().into()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn authorities() -> Vec<AuraId> {
            Aura::authorities()
        }

        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
        }
    }

    #[cfg(not(feature = "standalone"))]
    impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
        fn collect_collation_info() -> cumulus_primitives_core::CollationInfo {
            ParachainSystem::collect_collation_info()
        }
    }

    #[cfg(feature = "standalone")]
    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            _authority_id: GrandpaId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            // NOTE: this is the only implementation possible since we've
            // defined our key owner proof type as a bottom type (i.e. a type
            // with no values).
            None
        }

        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            _key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            None
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

            use frame_system_benchmarking::Module as SystemBench;
            impl frame_system_benchmarking::Config for Runtime {}

            let whitelist: Vec<TrackedStorageKey> = vec![
                // Block Number
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
                // Total Issuance
                hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
                // Execution Phase
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
                // Event Count
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
                // System Events
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
            ];

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);

            add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
            add_benchmark!(params, batches, pallet_timestamp, Timestamp);

            if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
            Ok(batches)
        }
    }
}

#[cfg(not(feature = "standalone"))]
cumulus_pallet_parachain_system::register_validate_block!(
    Runtime,
    cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
);
