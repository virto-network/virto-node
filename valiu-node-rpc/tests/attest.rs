#![cfg(feature = "_integration-tests")]

use sp_keyring::AccountKeyring;
use substrate_subxt::{extrinsic::PairSigner, ClientBuilder};
use valiu_node_commons::{Asset, Collateral};
use valiu_node_rpc::{AttestCallExt, ValiuRuntime};

#[tokio::test]
async fn attest() {
    let _ = env_logger::builder().is_test(true).try_init();

    let signer = PairSigner::new(AccountKeyring::Alice.pair());

    let client = ClientBuilder::<ValiuRuntime>::new().build().await.unwrap();

    let asset = Asset::Collateral(Collateral::Usdc);

    let rslt = client
        .attest_and_watch(&signer, asset, 123, Default::default())
        .await;

    assert!(rslt.is_err());
}
