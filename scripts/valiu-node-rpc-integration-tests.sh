#!/usr/bin/env bash

set -uxo pipefail

cargo build --release
cargo test --manifest-path valiu-node-rpc/Cargo.toml --no-run
cargo run --release -- purge-chain --dev -y
cargo run --release -- --dev & node_pid=$!
sleep 3
cargo test --features _integration-tests --manifest-path valiu-node-rpc/Cargo.toml
kill -9 $node_pid
