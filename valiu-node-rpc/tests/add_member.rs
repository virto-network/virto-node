#![cfg(feature = "_integration-tests")]

use sp_keyring::AccountKeyring;
use substrate_subxt::{extrinsic::PairSigner, ClientBuilder};
use valiu_node_rpc::{AddMemberCall, ValiuRuntime};

#[tokio::test]
async fn add_member() {
    let signer = PairSigner::new(AccountKeyring::Alice.pair());

    let client = ClientBuilder::<ValiuRuntime>::new().build().await.unwrap();

    let encoded = client
        .encode(AddMemberCall {
            origin: AccountKeyring::Alice.to_account_id(),
            who: AccountKeyring::Bob.to_account_id(),
        })
        .unwrap();

    let signed = client.create_signed(encoded, &signer).await.unwrap();

    let _ = client.submit_extrinsic(signed).await.unwrap();
}
