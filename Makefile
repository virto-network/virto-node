build-dev:
	cargo build -p vln-parachain

build-dev-node:
	cargo build -p vln-node

build-release:
	cargo build --release -p vln-parachain

build-release-node:
	cargo build --release -p vln-node

run-parachain-env:
	./scripts/parachain-dev-setup.sh

run-dev-node:
	./target/debug/vln_node --dev --tmp

test:
	./scripts/tests.sh

test-parachain:
	cargo test -p vln-parachain

test-node:
	cargo test -p vln-node