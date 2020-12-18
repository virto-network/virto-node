use crate::Membership;
use substrate_subxt::{balances::Balances, extrinsic::DefaultExtra, system::System, Runtime};
use valiu_node_runtime_types::{
    AccountData, AccountId, Address, Balance, BlockNumber, Hash, Hashing, Header, Index,
    OpaqueExtrinsic, Signature,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValiuRuntime;

impl Balances for ValiuRuntime {
    type Balance = Balance;
}

impl Membership for ValiuRuntime {}

impl Runtime for ValiuRuntime {
    type Extra = DefaultExtra<Self>;
    type Signature = Signature;
}

impl System for ValiuRuntime {
    type AccountData = AccountData;
    type AccountId = AccountId;
    type Address = Address;
    type BlockNumber = BlockNumber;
    type Extrinsic = OpaqueExtrinsic;
    type Hash = Hash;
    type Hashing = Hashing;
    type Header = Header;
    type Index = Index;
}
