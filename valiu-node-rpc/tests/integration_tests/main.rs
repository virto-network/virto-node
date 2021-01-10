#![cfg(feature = "_integration-tests")]

mod add_bob_as_a_member;
mod attest;
mod liquidity_provider_rpc;
mod members;
mod tokens;
mod transfer;

use sp_core::sr25519::Pair;
use sp_keyring::AccountKeyring;
use substrate_subxt::{extrinsic::PairSigner, Client, ClientBuilder};
use valiu_node_rpc::ValiuRuntime;

#[tokio::test]
async fn sequential_tests() {
    members::members().await;

    let (alice_pair, bob_pair, mut client) = initial().await;
    add_bob_as_a_member::add_bob_as_a_member(&mut client, &alice_pair).await;
    attest::attest(&mut client, &bob_pair).await;
    transfer::transfer(&mut client, &alice_pair, &bob_pair).await;
}

async fn initial() -> (
    PairSigner<ValiuRuntime, Pair>,
    PairSigner<ValiuRuntime, Pair>,
    Client<ValiuRuntime>,
) {
    let _ = env_logger::builder().is_test(true).try_init();
    let alice_pair = PairSigner::new(AccountKeyring::Alice.pair());
    let bob_pair = PairSigner::new(AccountKeyring::Bob.pair());
    let client = ClientBuilder::<ValiuRuntime>::new().build().await.unwrap();
    (alice_pair, bob_pair, client)
}
