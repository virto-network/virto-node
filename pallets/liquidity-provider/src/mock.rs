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
use valiu_node_commons::ValiuCurrencies;

impl_outer_origin! {
    pub enum Origin for Test {}
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type PalletInfo = ();
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
}

impl orml_tokens::Trait for Test {
    type Event = ();
    type Balance = u32;
    type Amount = i64;
    type CurrencyId = ValiuCurrencies;
    type OnReceived = ();
    type WeightInfo = ();
}

pub type Tokens = orml_tokens::Module<Test>;

ord_parameter_types! {
    pub const Root: u64 = 1;
}

impl pallet_membership::Trait for Test {
    type Event = ();
    type AddOrigin = EnsureSignedBy<Root, u64>;
    type RemoveOrigin = EnsureSignedBy<Root, u64>;
    type SwapOrigin = EnsureSignedBy<Root, u64>;
    type ResetOrigin = EnsureSignedBy<Root, u64>;
    type PrimeOrigin = EnsureSignedBy<Root, u64>;
    type MembershipInitialized = ();
    type MembershipChanged = ();
}
pub type Membership = pallet_membership::Module<Test>;

impl Trait for Test {
    type Event = ();
    type Currency = Tokens;
}

pub type TestProvider = Module<Test>;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
