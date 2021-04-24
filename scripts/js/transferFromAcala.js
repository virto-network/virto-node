// Forked from https://github.com/Phala-Network/phala-blockchain/blob/master/scripts/js/transferFromAcala.js
require("dotenv").config();
const { Keyring } = require("@polkadot/api");
const { cryptoWaitReady } = require("@polkadot/util-crypto");
const { buildConnection } = require("./connection.js");
const BN = require("bn.js");

const bn1e13 = new BN(10).pow(new BN(13));
const vlnParaId = 3586;
const acalaParaId = 1000;

const main = async () => {
  const api = await buildConnection("localacala");
  const acalaAccount = process.env.FROM || "//Alice";
  const vlnAccount = process.env.TO || "//Alice";
  const amount = new BN(process.env.AMOUNT);

  await cryptoWaitReady();

  const keyring = new Keyring({ type: "sr25519" });
  const sender = keyring.addFromUri(acalaAccount);
  const receiver = keyring.addFromUri(vlnAccount);
  let nonce = (await api.query.system.account(sender.address)).nonce.toNumber();

  const transfer = () => {
    return new Promise(async (resolve) => {
      const unsub = await api.tx.xTokens
        .transferToParachain(
          api.createType("XCurrencyId", {
            chainId: api.createType("ChainId", {
              Parachain: api.createType("Compact<U32>", acalaParaId),
            }),
            currencyId: "ACA",
          }),
          api.createType("Compact<U32>", vlnParaId),
          api.createType("MultiLocation", {
            X1: api.createType("Junction", {
              AccountId32: api.createType("AccountId32Junction", {
                network: api.createType("NetworkId", "Any"),
                id: receiver.address,
              }),
            }),
          }),
          api.createType("Compact<U128>", bn1e13.mul(amount))
        )
        .signAndSend(sender, { nonce: nonce, era: 0 }, (result) => {
          console.log(`Current status is ${result.status}`);
          if (result.status.isInBlock) {
            console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
          } else if (result.status.isFinalized) {
            console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
            unsub();
            resolve();
          }
        });
    });
  };

  await transfer();
  console.log("--- Transfer from Acala to VLN ---");
  console.log(`---   From: ${acalaAccount}`);
  console.log(`---     To: ${vlnAccount}`);
  console.log(`--- Amount: ${amount.toString()}`);
};

main()
  .catch(console.error)
  .finally(() => process.exit());
