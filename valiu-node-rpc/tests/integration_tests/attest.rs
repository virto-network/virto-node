use sp_core::sr25519::Pair;
use substrate_subxt::{extrinsic::PairSigner, Client};
use valiu_node_commons::{Asset, Collateral};
use valiu_node_rpc::{AttestCallExt, ValiuRuntime};

pub async fn attest(c: &mut Client<ValiuRuntime>, bob_pair: &PairSigner<ValiuRuntime, Pair>) {
    let asset = Asset::Collateral(Collateral::Usdc);
    let rslt = c
        .attest_and_watch(bob_pair, asset, 1, vec![])
        .await
        .unwrap();
    assert!(rslt.events.iter().any(|e| e.variant == "Attestation"));
}
