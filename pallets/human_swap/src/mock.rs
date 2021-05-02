use crate as human_swap;
use frame_support::{
    parameter_types,
    traits::{Contains, GenesisBuild},
};
use frame_system as system;
use orml_traits::{parameter_type_with_key, DataProvider};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    FixedU128,
};
use vln_primitives::DefaultRateCombinator;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
pub type AccountId = u8;
pub const PROVIDER_ONE: AccountId = 10;
pub const PROVIDER_TWO: AccountId = 11;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Tokens: orml_tokens::{Pallet, Call, Config<T>, Storage, Event<T>},
        RatePallet: vln_rate_provider::{Pallet, Call, Storage, Event<T>},
        HumanSwap: human_swap::{Pallet, Call, Storage, Event<T>}
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
            &PROVIDER_ONE | &PROVIDER_TWO => true,
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

parameter_type_with_key! {
    pub ExistentialDeposits: |currency_id: u32| -> u32 {
        0u32
    };
}

impl orml_tokens::Config for Test {
    type Amount = i64;
    type Balance = u32;
    type CurrencyId = u32;
    type Event = Event;
    type ExistentialDeposits = ExistentialDeposits;
    type OnDust = ();
    type WeightInfo = ();
}

impl vln_rate_provider::Config for Test {
    type Event = Event;
    type Asset = u32;
    type BaseAsset = u32;
    type Whitelist = MockMembership;
    type PriceFeed = MockOracle;
    type OracleValue = OracleValue;
    type RateCombinator = DefaultRateCombinator;
}

impl human_swap::Config for Test {
    type Event = Event;
    type Asset = Tokens;
    type RateProvider = RatePallet;
    type Whitelist = MockMembership;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    orml_tokens::GenesisConfig::<Test> {
        endowed_accounts: vec![(1, 2, 100), (PROVIDER_ONE, 2, 100)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}
