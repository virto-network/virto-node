use crate::initial;
use valiu_node_rpc::MembersCallExt;

pub async fn members() {
    let (alice_pair, _, c) = initial().await;
    let rslt = c.members_and_watch(&alice_pair).await.unwrap();
    assert!(rslt.events.iter().any(|e| e.variant == "Members"));
}
