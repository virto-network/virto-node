mod test_extrinsic;

use crate as pallet_liquidity_provider;
use crate::{mock::test_extrinsic::TestXt, Call, DefaultWeightInfo, Module, Trait};
use alloc::{boxed::Box, vec};
use frame_support::{
    impl_outer_event, impl_outer_origin, ord_parameter_types, parameter_types, weights::Weight,
};
use frame_system::EnsureRoot;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    traits::{IdentifyAccount, Verify},
    Perbill,
};
use vln_commons::{Asset, Collateral};

pub const USD_ASSET: Asset = Asset::Collateral(USD_COLLATERAL);
pub const USD_COLLATERAL: Collateral = Collateral::Usd;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Signature = sp_core::sr25519::Signature;
type BlockNumber = u64;
type Amount = i64;
type Balance = u64;
type Hash = sp_core::H256;
type Index = u32;

pub type Extrinsic = TestXt<AccountId, Call<Test>, ()>;
pub type ProviderMembers = pallet_membership::Module<Test, pallet_membership::DefaultInstance>;
pub type TestProvider = Module<Test>;
pub type Tokens = orml_tokens::Module<Test>;

impl_outer_event! {
    pub enum TestEvent for Test {
        frame_system<T>,
        orml_tokens<T>,
        pallet_membership<T>,
        pallet_liquidity_provider<T>,
    }
}

impl_outer_origin! {
    pub enum Origin for Test {}
}

ord_parameter_types! {
    pub const Root: AccountId = <AccountId>::from_raw([0; 32]);
}

parameter_types! {
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const BlockHashCount: BlockNumber = 250;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const MaximumBlockWeight: Weight = 1024;
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

impl frame_system::Trait for Test {
    type AccountData = ();
    type AccountId = AccountId;
    type AvailableBlockRatio = AvailableBlockRatio;
    type BaseCallFilter = ();
    type BlockExecutionWeight = ();
    type BlockHashCount = BlockHashCount;
    type BlockNumber = BlockNumber;
    type Call = ();
    type DbWeight = ();
    type Event = TestEvent;
    type ExtrinsicBaseWeight = ();
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type Index = Index;
    type Lookup = IdentityLookup<AccountId>;
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

impl<LC> frame_system::offchain::SendTransactionTypes<LC> for Test
where
    Call<Test>: From<LC>,
{
    type Extrinsic = Extrinsic;
    type OverarchingCall = Call<Test>;
}

impl orml_tokens::Trait for Test {
    type Amount = Amount;
    type Balance = Balance;
    type CurrencyId = Asset;
    type Event = TestEvent;
    type OnReceived = ();
    type WeightInfo = ();
}

impl pallet_membership::Trait<pallet_membership::DefaultInstance> for Test {
    type AddOrigin = EnsureRoot<AccountId>;
    type Event = TestEvent;
    type MembershipChanged = ();
    type MembershipInitialized = ();
    type PrimeOrigin = EnsureRoot<AccountId>;
    type RemoveOrigin = EnsureRoot<AccountId>;
    type ResetOrigin = EnsureRoot<AccountId>;
    type SwapOrigin = EnsureRoot<AccountId>;
}

impl Trait for Test {
    type Asset = Tokens;
    type Collateral = Tokens;
    type Event = TestEvent;
    type WeightInfo = DefaultWeightInfo;
}
