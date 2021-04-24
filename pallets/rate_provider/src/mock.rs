use crate as rate_provider;
use frame_support::{parameter_types, traits::Contains};
use frame_system as system;
use orml_traits::DataProvider;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    FixedU128,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
pub type AccountId = u8;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Rates: rate_provider::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
}

pub struct MockMembership;
impl Contains<AccountId> for MockMembership {
    fn contains(t: &AccountId) -> bool {
        match t {
            10 => true,
            _ => false,
        }
    }

    fn sorted_members() -> Vec<AccountId> {
        vec![]
    }
}

pub type OracleValue = FixedU128;
pub struct MockOracle;
impl DataProvider<u32, OracleValue> for MockOracle {
    fn get(key: &u32) -> Option<OracleValue> {
        match key {
            1 => Some(FixedU128::from(100)),
            _ => None,
        }
    }
}

impl rate_provider::Config for Test {
    type Event = Event;
    type Asset = u32;
    type BaseAsset = u32;
    type Whitelist = MockMembership;
    type PriceFeed = MockOracle;
    type OracleValue = OracleValue;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
