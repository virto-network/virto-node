#![cfg(feature = "_integration-tests")]
use substrate_subxt::ClientBuilder;
use vln_commons::Asset;
use vln_rpc::{TotalIssuanceStoreExt, VlnRuntime};

#[tokio::test]
async fn token_issuance() {
    let _ = env_logger::builder().is_test(true).try_init();

    let client = ClientBuilder::<VlnRuntime>::new().build().await.unwrap();
    let issued = client.total_issuance(Asset::Btc, None).await.unwrap();
    assert_eq!(issued, 0);
}
