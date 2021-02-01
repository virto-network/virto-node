#!/usr/bin/env bash

set -euxo pipefail

export RUSTFLAGS='
    -D bad_style
    -D future_incompatible
    -D missing_debug_implementations
    -D nonstandard_style
    -D rust_2018_compatibility
    -D rust_2018_idioms
    -D unused_qualifications
    -D warnings
'

test_package_with_feature() {
    local package=$1
    local features=$2

    cargo test --features $features --manifest-path $package/Cargo.toml --no-default-features
}

cargo fmt --all -- --check
cargo clippy --all-features

test_package_with_feature vln-commons std

test_package_with_feature vln-pallets/liquidity-provider native-runtime-benchmarks
test_package_with_feature vln-pallets/liquidity-provider std

test_package_with_feature vln-runtime native-runtime-benchmarks
test_package_with_feature vln-runtime std

test_package_with_feature vln-node default
test_package_with_feature vln-node native-runtime-benchmarks
