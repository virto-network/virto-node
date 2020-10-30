use crate::{Module, Trait};
use frame_support::{impl_outer_origin, ord_parameter_types, parameter_types, weights::Weight};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};
use system::EnsureSignedBy;
use valiu_node_commons::Asset;

pub type ProviderMembers = pallet_membership::Module<Test, pallet_membership::DefaultInstance>;
pub type TestProvider = Module<Test>;
pub type Tokens = orml_tokens::Module<Test>;

impl_outer_origin! {
    pub enum Origin for Test {}
}

ord_parameter_types! {
    pub const Root: u64 = 1;
}

parameter_types! {
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const MaximumBlockWeight: Weight = 1024;
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

impl system::Trait for Test {
    type AccountData = ();
    type AccountId = u64;
    type AvailableBlockRatio = AvailableBlockRatio;
    type BaseCallFilter = ();
    type BlockExecutionWeight = ();
    type BlockHashCount = BlockHashCount;
    type BlockNumber = u64;
    type Call = ();
    type DbWeight = ();
    type Event = ();
    type ExtrinsicBaseWeight = ();
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type Index = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type MaximumBlockLength = MaximumBlockLength;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type Origin = Origin;
    type PalletInfo = ();
    type SystemWeightInfo = ();
    type Version = ();
}

impl orml_tokens::Trait for Test {
    type Amount = i64;
    type Balance = u32;
    type CurrencyId = Asset;
    type Event = ();
    type OnReceived = ();
    type WeightInfo = ();
}

impl pallet_membership::Trait<pallet_membership::DefaultInstance> for Test {
    type AddOrigin = EnsureSignedBy<Root, u64>;
    type Event = ();
    type MembershipChanged = ();
    type MembershipInitialized = ();
    type PrimeOrigin = EnsureSignedBy<Root, u64>;
    type RemoveOrigin = EnsureSignedBy<Root, u64>;
    type ResetOrigin = EnsureSignedBy<Root, u64>;
    type SwapOrigin = EnsureSignedBy<Root, u64>;
}

impl Trait for Test {
    type Asset = Tokens;
    type Collateral = Tokens;
    type Event = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
