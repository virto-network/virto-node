#![cfg(feature = "_integration-tests")]

pub mod attest;
pub mod transfer;

use sp_core::sr25519::Pair;
use sp_keyring::AccountKeyring;
use substrate_subxt::{extrinsic::PairSigner, sudo::SudoCallExt, Client};
use valiu_node_rpc::{AddMemberCall, ValiuRuntime};

async fn add_bob_as_a_member(
    cb: &mut Client<ValiuRuntime>,
    signer: &PairSigner<ValiuRuntime, Pair>,
) {
    let encoded = cb
        .encode(AddMemberCall {
            who: AccountKeyring::Bob.public(),
        })
        .unwrap();
    let _ = cb.sudo_and_watch(signer, &encoded).await;
}
