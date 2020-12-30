#![cfg(feature = "_integration-tests")]

use sp_keyring::AccountKeyring;
use substrate_subxt::{extrinsic::PairSigner, ClientBuilder};
use valiu_node_commons::DistributionStrategy;
use valiu_node_rpc::{TransferCallExt, ValiuRuntime};

#[tokio::test]
async fn transfer() {
    let _ = env_logger::builder().is_test(true).try_init();

    let signer = PairSigner::new(AccountKeyring::Alice.pair());

    let client = ClientBuilder::<ValiuRuntime>::new().build().await.unwrap();

    let rslt = client
        .transfer_and_watch(
            &signer,
            AccountKeyring::Bob.public(),
            1,
            DistributionStrategy::Evenly,
        )
        .await;

    assert!(rslt.is_err());
}
