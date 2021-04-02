#!/bin/bash
# Spinup a local env with relaychains and parachains for testing.
# This script expects compiled polkadot and vln binaries under /polkadot and /vln-node
# directories. Run this script only after initial setup as per https://github.com/paritytech/cumulus/blob/master/README.md

set -e
# turn a string into a flag
function flagify() {
  printf -- '--%s' "$(tr '[:upper:]' '[:lower:]' <<< "$1")"
}

# start relaychains in seperate screens for debugging
function startrelaychain() {
 screen_name="$1"
 auth="$2"
 port="$3"
 wsport="$4"
 rpcport="$5"

 screen -dmS "$screen_name" \
 ../polkadot/target/release/polkadot --chain "../polkadot/rococo-local-cfde-real-overseer.json" \
 --tmp --rpc-external --ws-external --rpc-cors all --discover-local -lruntime=trace \
  --ws-port "$wsport" --port "$port" --rpc-port "$rpcport" \
  "$(flagify "$auth")"
}

function startvlnparachain() {
 screen_name="$1"
 chain="$2"
 port="$3"
 wsport="$4"
 rpcport="$5"

 # create outputs for chainid
 ./target/debug/vln_parachain export-genesis-state --parachain-id "$chain" > genesis-state-"$chain"

 ./target/debug/vln_parachain export-genesis-wasm > genesis-wasm-"$chain"

 screen -dmS "$screen_name" \
 ./target/debug/vln_parachain --collator --tmp \
 --parachain-id "$chain" -lruntime=trace \
 --rpc-external --ws-external --rpc-cors all \
 --port "$port" --ws-port "$wsport" -- --execution wasm \
 --chain "../polkadot/rococo-local-cfde-real-overseer.json" \
 --port "$rpcport"
}

# Start relaychain
startrelaychain relay1 alice 30333 9944 9933
startrelaychain relay2 bob 30334 9945 9934
startrelaychain relay3 charlie 30335 9946 9935

# start parachain
startvlnparachain vln 200 40335 9947 30336
#startvlnparachain vln2 400 40336 9948 30337