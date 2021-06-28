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

cargo fmt --all -- --check
#cargo clippy --exclude vln-parachain --workspace // save test time
cargo clippy -p vln-parachain # testing seperately since feature flags leads to conflict
#cargo test --exclude vln-parachain --workspace // save test time
cargo test -p vln-parachain # testing seperately since feature flags leads to conflict
