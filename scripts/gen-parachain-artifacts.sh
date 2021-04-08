#!/usr/bin/env bash

set -e

./target/release/vln_parachain export-genesis-state --chain testnet > genesis-state
./target/release/vln_parachain export-genesis-wasm --chain testnet > genesis-wasm