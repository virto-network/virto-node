use sp_core::sr25519::Pair;
use substrate_subxt::{extrinsic::PairSigner, Client};
use valiu_node_commons::{Asset, Collateral};
use valiu_node_rpc::{AttestCallExt, TransferCallExt, ValiuRuntime};

pub async fn transfer(
    c: &mut Client<ValiuRuntime>,
    alice_pair: &PairSigner<ValiuRuntime, Pair>,
    bob_pair: &PairSigner<ValiuRuntime, Pair>,
) {
    use sp_core::Pair;
    let asset = Asset::Collateral(Collateral::Usdc);
    let _ = c
        .attest_and_watch(bob_pair, asset, 1, vec![])
        .await
        .unwrap();
    let rslt = c
        .transfer_and_watch(bob_pair, alice_pair.signer().public(), 1)
        .await
        .unwrap();
    assert!(rslt.events.iter().any(|e| e.variant == "Transfer"));
}
