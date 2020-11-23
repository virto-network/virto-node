#!/usr/bin/env bash

set -eux

check_package_with_feature() {
    local package=$1
    local features=$2

    cargo check --manifest-path "${package}"/Cargo.toml --features "${features}" --no-default-features
}

check_package_with_feature valiu-node-commons default
check_package_with_feature valiu-node-commons std

check_package_with_feature pallets/liquidity-provider native-runtime-benchmarks
check_package_with_feature pallets/liquidity-provider std

check_package_with_feature runtime native-runtime-benchmarks
check_package_with_feature runtime std

check_package_with_feature node default
check_package_with_feature node native-runtime-benchmarks

