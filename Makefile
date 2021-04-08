.PHONY: build-dev
build-dev:
	cargo build -p vln-parachain

.PHONY: build-dev-node
build-dev-node:
	cargo build -p vln-node

.PHONY: build-release
build-release:
	cargo build --release -p vln-parachain

.PHONY: build-release-node
build-release-node:
	cargo build --release -p vln-node

.PHONY: run-parachain-env
run-parachain-env:
	./scripts/parachain-dev-setup.sh

.PHONY: run-dev-node
run-dev-node:
	./target/debug/vln_node --dev --tmp

.PHONY: test
test:
	./scripts/tests.sh

.PHONY: test-parachain
test-parachain:
	cargo test -p vln-parachain

.PHONY: test-node
test-node:
	cargo test -p vln-node