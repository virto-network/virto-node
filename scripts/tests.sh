#!/usr/bin/env bash

set -euxo pipefail

#-D future_incompatible
export RUSTFLAGS='
    -D bad_style
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
cargo clippy --exclude vln-parachain --workspace
cargo clippy -p vln-parachain # testing seperately since feature flags leads to conflict

# NOTE: After update there's an issue of a dependency not compiling to WASM
#test_package_with_feature runtime native-runtime-benchmarks
test_package_with_feature runtime std

test_package_with_feature node/standalone default
#test_package_with_feature node native-runtime-benchmarks
