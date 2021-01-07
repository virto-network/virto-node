use crate::{LiquidityProvider, ProviderMembers, Tokens, ValiuExtra};
use substrate_subxt::{sudo::Sudo, system::System, Runtime};
use valiu_node_commons::{Asset, Collateral};
use valiu_node_runtime_types::{
    AccountData, AccountId, Amount, Balance, BlockNumber, Hash, Hashing, Header, Index,
    OpaqueExtrinsic, Signature,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValiuRuntime;

impl LiquidityProvider for ValiuRuntime {
    type Asset = Asset;
    type Collateral = Collateral;
}

impl ProviderMembers for ValiuRuntime {}

impl Runtime for ValiuRuntime {
    type Extra = ValiuExtra<Self>;
    type Signature = Signature;
}

impl Sudo for ValiuRuntime {}

impl System for ValiuRuntime {
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

impl Tokens for ValiuRuntime {
    type Amount = Amount;
    type Balance = Balance;
    type CurrencyId = Asset;
}
