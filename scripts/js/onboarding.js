// setup and onboard a local parachain test env
// and test asset transfer
// switching to live relay should allow you to perform most of these actions
// on Rococo, but most of these havent been tested in that scenario yet
const { registerParachainOnLocalRelay } = require("./register_parachain");
const shell = require("shelljs");
const { openHrmpChanneltoVln, acceptHrmpRequestfromAcala } = require("./open_hrmp_channel");

async function main() {
  // Comment out steps you dont require
  // STEP 1 : Setup a local environment for testing
  // The scripts that follow are tied to the ports and file locations used in this script
  // Remember to modify accordingly everywhere
  shell.exec("../parachain-dev-setup.sh");

  // STEP 2 : Register parachains on the relay chain
  // Register VLN to the relay chain
  console.log("Registering VLN on relay chain");
  await registerParachainOnLocalRelay(3586, "./genesis-state-3586", "./genesis-wasm-3586");
  // Register acala to the relay chain
  console.log("Registering Acala on relay chain");
  await registerParachainOnLocalRelay(
    1000,
    "../../../Acala/genesis-state-1000",
    "../../../Acala/genesis-wasm-1000"
  );

  // STEP 3 : Open an hrmp channel between acala and VLN
  // generate request from acala to vln
  console.log("Opening hrmp request from acala..");
  await openHrmpChanneltoVln(3586, "localrelay", "localacala");

  // accept request from VLN
  console.log("accepting hrmp request from vln..");
  await acceptHrmpRequestfromAcala(1000, "localrelay", "localvln");
}

main()
  .catch(console.error)
  .finally(() => process.exit());
