#!/usr/bin/env bash

set -euxo pipefail

cargo build --release
cargo test --manifest-path valiu-node-rpc/Cargo.toml --no-run
cargo run --release -- --dev &
sleep 1
cargo test --manifest-path valiu-node-rpc/Cargo.toml
pkill vln_node
