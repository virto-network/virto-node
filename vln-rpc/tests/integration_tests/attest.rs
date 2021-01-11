use sp_core::sr25519::Pair;
use substrate_subxt::{extrinsic::PairSigner, Client};
use vln_commons::{Asset, Collateral};
use vln_rpc::{AttestCallExt, VlnRuntime};

pub async fn attest(c: &mut Client<VlnRuntime>, bob_pair: &PairSigner<VlnRuntime, Pair>) {
    let asset = Asset::Collateral(Collateral::Usdc);
    let rslt = c
        .attest_and_watch(bob_pair, asset, 1, vec![])
        .await
        .unwrap();
    assert!(rslt.events.iter().any(|e| e.variant == "Attestation"));
}
