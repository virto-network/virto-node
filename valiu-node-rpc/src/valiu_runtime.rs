use crate::{LiquidityProvider, ProviderMembers, ValiuExtra, Tokens};
use substrate_subxt::{balances::Balances, sudo::Sudo, system::System, Runtime};
use valiu_node_runtime_types::{
    AccountData, AccountId, Balance, BlockNumber, Hash, Hashing, Header, Index, OpaqueExtrinsic,
    Signature,
};
use valiu_node_commons::Asset;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValiuRuntime;

impl Balances for ValiuRuntime {
    type Balance = Balance;
}

impl LiquidityProvider for ValiuRuntime {}

impl ProviderMembers for ValiuRuntime {}

impl Tokens for ValiuRuntime {
    type Amount = i64;
    type Balance = Balance;
    type CurrencyId = Asset;
}

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
