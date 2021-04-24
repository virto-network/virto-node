// Register as a parathread on relay chain
// Ensure the account used has enough balance on the relay chain specified
const { Keyring } = require('@polkadot/api');
const { sendSudoCall } = require('./open_hrmp_channel');
const { buildConnection } = require("./connection.js");
require('dotenv').config();

function registerParachainOnLocalRelay(chain_id, genesisHead, validationCode) {
    return new Promise(async (resolve) => {
        const api = await buildConnection("localrelay");
        const proposal = api.tx.parasSudoWrapper.sudoScheduleParaInitialize(chain_id, { genesisHead, validationCode, parachain: true });
        const keyring = new Keyring({ type: "sr25519" });
        const adminPair = keyring.addFromUri("//Alice");
        await sendSudoCall(api, proposal, adminPair);
        resolve();
    });
}

async function registerParachainOnRococo(chain_id, genesisHead, validationCode) {
    const api = await buildConnection("liverelay");
    const keyring = new Keyring({ type: "sr25519" });
    const senderPair = keyring.addFromUri(process.env.ROC_ACCOUNT); // ensure this accout has enough ROC (~150) to pull this off 
    return api.tx.parasSudoWrapper.sudoScheduleParaInitialize(chain_id, { genesisHead, validationCode, parachain: true }).signAndSend(senderPair);
}

module.exports = {
    registerParachainOnLocalRelay,
    registerParachainOnRococo,
  };