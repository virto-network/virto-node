use crate::{Call, Module, Trait};
use frame_support::{impl_outer_origin, ord_parameter_types, parameter_types, weights::Weight};
use frame_system::{offchain::AppCrypto, EnsureSignedBy};
use sp_core::{sr25519, H256};
use sp_runtime::{
    generic::Header,
    testing::TestXt,
    traits::Verify,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};
use valiu_node_commons::Asset;

pub type Extrinsic = TestXt<Call<Test>, ()>;
pub type ProviderMembers = pallet_membership::Module<Test, pallet_membership::DefaultInstance>;
pub type TestProvider = Module<Test>;
pub type Tokens = orml_tokens::Module<Test>;

impl_outer_origin! {
    pub enum Origin for Test where system = frame_system {}
}

ord_parameter_types! {
    pub const Root: sr25519::Public = <sr25519::Public>::from_raw([0; 32]);
}

parameter_types! {
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const OffchainUnsignedGracePeriod: u64 = 5;
    pub const OffchainUnsignedInterval: u64 = 128;
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

impl frame_system::Trait for Test {
    type AccountData = ();
    type AccountId = sr25519::Public;
    type AvailableBlockRatio = AvailableBlockRatio;
    type BaseCallFilter = ();
    type BlockExecutionWeight = ();
    type BlockHashCount = BlockHashCount;
    type BlockNumber = u128;
    type Call = ();
    type DbWeight = ();
    type Event = ();
    type ExtrinsicBaseWeight = ();
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Header = Header<u128, BlakeTwo256>;
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

impl<LC> frame_system::offchain::SendTransactionTypes<LC> for Test
where
    Call<Test>: From<LC>,
{
    type Extrinsic = Extrinsic;
    type OverarchingCall = Call<Test>;
}

impl frame_system::offchain::SigningTypes for Test {
    type Public = <sr25519::Signature as Verify>::Signer;
    type Signature = sr25519::Signature;
}

impl orml_tokens::Trait for Test {
    type Amount = i128;
    type Balance = u128;
    type CurrencyId = Asset;
    type Event = ();
    type OnReceived = ();
    type WeightInfo = ();
}

impl pallet_membership::Trait<pallet_membership::DefaultInstance> for Test {
    type AddOrigin = EnsureSignedBy<Root, sr25519::Public>;
    type Event = ();
    type MembershipChanged = ();
    type MembershipInitialized = ();
    type PrimeOrigin = EnsureSignedBy<Root, sr25519::Public>;
    type RemoveOrigin = EnsureSignedBy<Root, sr25519::Public>;
    type ResetOrigin = EnsureSignedBy<Root, sr25519::Public>;
    type SwapOrigin = EnsureSignedBy<Root, sr25519::Public>;
}

impl Trait for Test {
    type Asset = Tokens;
    type Collateral = Tokens;
    type Event = ();
    type OffchainAuthority = TestAuth;
    type OffchainUnsignedGracePeriod = OffchainUnsignedGracePeriod;
    type OffchainUnsignedInterval = OffchainUnsignedInterval;
}

pub struct TestAuth;

impl AppCrypto<<sr25519::Signature as Verify>::Signer, sr25519::Signature> for TestAuth {
    type GenericPublic = sr25519::Public;
    type GenericSignature = sr25519::Signature;
    type RuntimeAppPublic = crate::Public;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
