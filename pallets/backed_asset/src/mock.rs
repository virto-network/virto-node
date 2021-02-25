use crate as backed_asset;
use frame_support::{parameter_types, traits::GenesisBuild};
use frame_system as system;
use orml_traits::parameter_type_with_key;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
pub type CurrencyId = u8;
pub type Balance = u32;
pub type AccountId = u8;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        Tokens: orml_tokens::{Module, Call, Config<T>, Storage, Event<T>},
        Asset: backed_asset::{Module, Call, Storage, Event<T>},
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
}

parameter_type_with_key! {
    pub ExistentialDeposits: |_id: CurrencyId| -> Balance { 0 };
}
impl orml_tokens::Config for Test {
    type Amount = i64;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type Event = Event;
    type ExistentialDeposits = ExistentialDeposits;
    type OnDust = ();
    type WeightInfo = ();
}

impl backed_asset::Config for Test {
    type Event = Event;
    type Collateral = Tokens;
    type BaseCurrency = orml_tokens::CurrencyAdapter<Test, ()>;
    type Balance = Balance;
}

pub fn new_test_with_accounts(
    accounts: &[(AccountId, CurrencyId, Balance)],
) -> sp_io::TestExternalities {
    let mut t = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    orml_tokens::GenesisConfig::<Test> {
        endowed_accounts: accounts.into(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}
