// Request opening of hrmp channel from acala to VLN
// Accept hrmp open channel request from VLN
const { Keyring } = require("@polkadot/api");
const { buildConnection } = require("./connection.js");

async function openHrmpChanneltoVln(chain_id, relay = "localrelay", parachain = "localacala") {
  // form the encoded txn
  let encoded_tx = await genHrmpOpenInit(chain_id, relay);
  console.log("Encoded Tx : ", encoded_tx);

  const api = await buildConnection(parachain);
  const keyring = new Keyring({ type: "sr25519" });
  const adminPair = await keyring.addFromUri("//Alice");

  // form transaction to create request
  const proposal = api.tx.xcmHandler.sendUpwardXcm(
    api.createType("VersionedXcm", {
      V0: api.createType("Xcm", {
        Transact: api.createType("XcmTransact", {
          originType: "Native",
        }),
        call: encoded_tx,
      }),
    })
  );

  return new Promise(async (resolve) => {
    // send sudo call to init request
    await sendSudoCall(api, proposal, adminPair);
    resolve();
  });
}

async function genHrmpOpenInit(chain_id, relay = "localrelay") {
  // following instructions here https://wiki.acala.network/build/development-guide/composable-chains/open-hrmp-channel
  const api = await buildConnection(relay);
  const tx = await api.tx.hrmp.hrmpInitOpenChannel(chain_id, 8, 1024);
  let encoded_tx = tx.toHex();
  return encoded_tx.split("3c04")[0] + encoded_tx.split("3c04")[1];
}

async function genHrmpAcceptMsg(chain_id, relay = "localrelay") {
  // following instructions here https://wiki.acala.network/build/development-guide/composable-chains/open-hrmp-channel
  const api = await buildConnection(relay);
  const tx = api.tx.hrmp.hrmpAcceptOpenChannel(chain_id);
  let encoded_tx = tx.toHex();
  return encoded_tx.split("1c04")[0] + encoded_tx.split("1c04")[1];
}

async function acceptHrmpRequestfromAcala(chain_id, relay = "localrelay", parachain = "localvln") {
  // form the encoded txn
  let encoded_tx = await genHrmpAcceptMsg(chain_id, relay);
  console.log("Encoded Tx : ", encoded_tx);

  const api = await buildConnection(parachain);
  const keyring = new Keyring({ type: "sr25519" });
  const adminPair = await keyring.addFromUri("//Alice");

  // form transaction to create request
  const proposal = api.tx.xcmHandler.sendUpwardXcm(
    api.createType("VersionedXcm", {
      V0: api.createType("Xcm", {
        Transact: api.createType("XcmTransact", {
          originType: "Native",
        }),
        call: encoded_tx,
      }),
    })
  );

  return new Promise(async (resolve) => {
    // send sudo call to init request
    await sendSudoCall(api, proposal, adminPair);
    resolve();
  });
}

function sendSudoCall(api, proposal, adminPair) {
  return api.tx.sudo.sudo(proposal).signAndSend(adminPair, ({ events = [], status }) => {
    console.log("Proposal status:", status.type);
    if (status.isInBlock) {
      console.log("Included at block hash", status.asInBlock.toHex());
      console.log("Events:");
      console.log(JSON.stringify(events, null, 2));
    } else if (status.isFinalized) {
      console.log("Finalized block hash", status.asFinalized.toHex());
    }
  });
}

module.exports = {
  openHrmpChanneltoVln,
  acceptHrmpRequestfromAcala,
  sendSudoCall,
};
