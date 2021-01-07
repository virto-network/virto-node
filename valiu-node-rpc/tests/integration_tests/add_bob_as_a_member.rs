use sp_core::{sr25519::Pair, Decode};
use sp_keyring::AccountKeyring;
use substrate_subxt::{extrinsic::PairSigner, sudo::SudoCallExt, Client};
use valiu_node_rpc::{AddMemberCall, MembersCallExt, MembersEvent, ValiuRuntime};

pub async fn add_bob_as_a_member(
    c: &mut Client<ValiuRuntime>,
    alice_pair: &PairSigner<ValiuRuntime, Pair>,
) {
    let bob_public = AccountKeyring::Bob.public();
    let encoded = c.encode(AddMemberCall { who: bob_public }).unwrap();
    let ext = c.members_and_watch(alice_pair).await.unwrap();
    let mut data = &ext.events.get(0).unwrap().data[..];
    let event = MembersEvent::<ValiuRuntime>::decode(&mut data).unwrap();
    if event.members.iter().all(|e| e != &bob_public) {
        assert!(c.sudo_and_watch(alice_pair, &encoded).await.is_ok());
    }
}
