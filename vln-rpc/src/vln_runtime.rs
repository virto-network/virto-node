use crate::{LiquidityProvider, ProviderMembers, Tokens, VlnExtra};
use substrate_subxt::{sudo::Sudo, system::System, Runtime};
use vln_commons::{
    runtime::{
        AccountData, AccountId, Amount, Balance, BlockNumber, Hash, Hashing, Header, Index,
        OpaqueExtrinsic, Signature,
    },
    Asset, Collateral,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VlnRuntime;

impl LiquidityProvider for VlnRuntime {
    type Asset = Asset;
    type Collateral = Collateral;
}

impl ProviderMembers for VlnRuntime {}

impl Runtime for VlnRuntime {
    type Extra = VlnExtra<Self>;
    type Signature = Signature;
}

impl Sudo for VlnRuntime {}

impl System for VlnRuntime {
    type AccountData = AccountData;
    type AccountId = AccountId;
    type Address = AccountId;
    type BlockNumber = BlockNumber;
    type Extrinsic = OpaqueExtrinsic;
    type Hash = Hash;
    type Hashing = Hashing;
    type Header = Header;
    type Index = Index;
}

impl Tokens for VlnRuntime {
    type Amount = Amount;
    type Balance = Balance;
    type CurrencyId = Asset;
}
