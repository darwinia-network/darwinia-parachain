#!/bin/sh

cargo clean -p darwinia-collator 2> /dev/null || true
cargo clean -p polkadot-runtime 2> /dev/null || true
cargo clean -p kusama-runtime 2> /dev/null || true
cargo clean -p darwinia-parachain-runtime 2> /dev/null || true
cargo clean -p crab-parachain-runtime 2> /dev/null || true
cargo clean -p pangolin-parachain-runtime 2> /dev/null || true
rm -rf target/debug/wbuild 2> /dev/null || true

cargo clean --release -p darwinia-collator 2> /dev/null || true
cargo clean --release -p polkadot-runtime 2> /dev/null || true
cargo clean --release -p kusama-runtime 2> /dev/null || true
cargo clean --release -p darwinia-parachain-runtime 2> /dev/null || true
cargo clean --release -p crab-parachain-runtime 2> /dev/null || true
cargo clean --release -p pangolin-parachain-runtime 2> /dev/null || true
rm -rf target/release/wbuild 2> /dev/null || true
