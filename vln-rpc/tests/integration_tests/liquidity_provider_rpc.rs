#![cfg(feature = "_integration-tests")]
use substrate_subxt::ClientBuilder;
use vln_commons::Asset;
use vln_rpc::{AccountRatesStoreExt, VlnRuntime};

#[tokio::test]
async fn get_account_rates() {
    let _ = env_logger::builder().is_test(true).try_init();
    let client = ClientBuilder::<VlnRuntime>::new().build().await.unwrap();
    let rates = client
        .account_rates(Asset::Btc, Asset::Usdv, None)
        .await
        .unwrap();
    assert_eq!(rates, vec![]);
}
