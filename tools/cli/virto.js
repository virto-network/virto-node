// Import the API
const { ApiPromise, WsProvider } = require("@polkadot/api");
const { Keyring } = require("@polkadot/keyring");
const { cryptoWaitReady } = require("@polkadot/util-crypto");
const yargs = require("yargs/yargs");
const { hideBin } = require("yargs/helpers");
const argv = yargs(hideBin(process.argv)).string("from").argv;

const NODE_ENDPOINT = "ws://127.0.0.1:9946";

/**
 * CLI commands
 * --action ['pay','release']
 * --from [privkey]
 * --to [address]
 * --assetid [string]
 * --amount [value]
 * --remark [string]
 * 
 To transfer from alice to Bob
 node virto.js --action=pay --to=5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty --assetid Network::KSM --amount=5 --from=0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a
*/

async function main() {
  // Create an await for the API
  const wsProvider = new WsProvider(NODE_ENDPOINT);
  const api = await ApiPromise.create({ provider: wsProvider });

  const keyring = new Keyring({ type: "sr25519", ss58Format: 42 });
  await cryptoWaitReady();
  const sp = keyring.createFromUri(argv.from, { name: "sr25519" });

  console.info(`########################################`);
  console.info(
    `Executing ${argv.action} from ${sp.address} to ${argv.to} for asset ${argv.assetid} amount ${argv.amount}`
  );

  if (argv.action == "pay") {
    const txHash = await api.tx.payment
      .pay(argv.recipient, argv.asset_id, argv.amount, argv.remark)
      .signAndSend(sp);
    console.log(`Submitted with hash : ${txHash}`);
  }

  else if (argv.action == "release") {
    const txHash = await api.tx.payment
      .release(argv.recipient)
      .signAndSend(sp);
    console.log(`Submitted with hash : ${txHash}`);
  }

  else if (argv.action == "cancel") {
    const txHash = await api.tx.payment
      .cancel(argv.recipient)
      .signAndSend(sp);
    console.log(`Submitted with hash : ${txHash}`);
  }

}

main().catch(console.error);
