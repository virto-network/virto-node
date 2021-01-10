#![cfg(feature = "_integration-tests")]
use substrate_subxt::ClientBuilder;
use valiu_node_commons::Asset;
use valiu_node_rpc::{AccountRatesStoreExt, ValiuRuntime};

#[tokio::test]
async fn get_account_rates() {
    let _ = env_logger::builder().is_test(true).try_init();
    let client = ClientBuilder::<ValiuRuntime>::new().build().await.unwrap();
    let rates = client
        .account_rates(Asset::Btc, Asset::Usdv, None)
        .await
        .unwrap();
    assert_eq!(rates, vec![]);
}
