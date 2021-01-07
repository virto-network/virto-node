use sp_core::sr25519::Pair;
use substrate_subxt::{extrinsic::PairSigner, Client};
use valiu_node_rpc::{TransferCallExt, ValiuRuntime};

pub async fn transfer(
    c: &mut Client<ValiuRuntime>,
    alice_pair: &PairSigner<ValiuRuntime, Pair>,
    bob_pair: &PairSigner<ValiuRuntime, Pair>,
) {
    use sp_core::Pair;
    let rslt = c
        .transfer_and_watch(bob_pair, alice_pair.signer().public(), 0)
        .await
        .unwrap();
    assert!(rslt.events.iter().any(|e| e.variant == "Transfer"));
}
