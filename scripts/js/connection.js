const { ApiPromise, WsProvider } = require("@polkadot/api");
const { options } = require("@acala-network/api");
const vln_types = require("../../types.json");
const { types } = require("@acala-network/types");

// these values align with test script@parachain-dev-setup, remember to modify in both places
const NETWORK_PROVIDER = {
  localrelay: "ws://127.0.0.1:9944",
  liverelay: "wss://rococo-rpc.polkadot.io",
  localvln: "ws://127.0.0.1:9947",
  livevln: "wss://vln.valiu.dev",
  localacala: "ws://127.0.0.1:9979",
  liveacala: "wss://rococo-1.acala.laminar.one",
};

const CUSTOM_TYPES = {
  relay: null,
  localvln: vln_types,
  livevln: vln_types,
};

function buildConnection(network) {
  if (!(network in NETWORK_PROVIDER)) throw new Error("Invalid Network!");
  const provider = new WsProvider(NETWORK_PROVIDER[network]);
  return network != "localacala"
    ? ApiPromise.create({
        provider,
        types: CUSTOM_TYPES[network],
      })
    : ApiPromise.create(options({ provider, types: types }));
}

module.exports = {
  buildConnection,
  NETWORK_PROVIDER,
};
