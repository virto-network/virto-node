chain?=testnet
# out directory
BUILD=build
BIN=vln
SRC_DIRS=node runtime pallets primitives
SRC_FILES=$(shell find $(SRC_DIRS) -type f)
DOCKER=$(shell which podman 2>/dev/null || which docker)
COMPOSE=$(shell which podman-compose 2>/dev/null || which docker-compose)

# passing a dev=yes argument builds in debug mode
BUILD_FLAGS=--release
ENV=release
ifeq ($(dev), yes)
	BUILD_FLAGS=
	ENV=debug
endif

# whether to build in parachain or stand-alone mode
MODES=parachain node
TARGET=$(MODES:%=target/$(ENV)/$(BIN)_%)
TEST=$(MODES:%=test_%)
CLIPPY=$(MODES:%=clippy_%)

.PHONY: build
build: $(BUILD)/$(BIN) $(BUILD)/$(chain)_$(BIN)_genesis_state \
	$(BUILD)/$(chain)_$(BIN)_genesis_wasm \
	$(BUILD)/$(chain)_$(BIN)_chainspec

# default bin is the parachain node
$(BUILD)/$(BIN): target/$(ENV)/$(BIN)_parachain
	@mkdir -p $(BUILD)
	@cp $< $@

$(BUILD)/$(chain)_$(BIN)_genesis_state: $(BUILD)/$(BIN)
	$^ export-genesis-state --chain $(chain) > $@

$(BUILD)/$(chain)_$(BIN)_genesis_wasm: $(BUILD)/$(BIN)
	$^ export-genesis-wasm --chain $(chain) > $@

$(BUILD)/$(chain)_$(BIN)_chainspec: $(BUILD)/$(BIN)
	$^ build-spec --chain $(chain) --disable-default-bootnode > $@

$(TARGET): $(SRC_FILES)
	cargo build $(BUILD_FLAGS) -p $(BIN)-$(@:target/$(ENV)/$(BIN)_%=%)

# Containerize application. It uses the already built binary(e.g. during CI)
# and puts it in a cointainer, since the target image is a debian based container
# this won't likely work unless run in a similar debian installation.
.PHONY: container
img?=valibre/vln
tag?=$(shell git describe --tags)
container: $(BUILD)/$(BIN)
	$(DOCKER) build . -t $(img):$(tag) -t $(img):latest \
		--build-arg VCS_REF=$(tag) \
		--build-arg IMAGE_NAME=$(img) \
		--build-arg BUILD_DATE=$(shell date +'%Y-%m-%d')

.PHONY: test
test: $(TEST)

.PHONY: check
check: $(CLIPPY)
	cargo fmt --all -- --check

# The substitution $(@:test_%=%) extracts "node" or "parachain"
# from the target that looks like test_node
$(TEST):
	cargo test -p $(BIN)-$(@:test_%=%)

$(CLIPPY):
	cargo clippy -p $(BIN)-parachain

#
# Testing parachain locally
# Run command sets up a "devnet" with relay-chain validators, karura and vln collators
#
.PHONY: run run-parachain stop
POLKADOT=parity/polkadot:v0.9.7
KARURA=acala/karura-node
VLN=valibre/vln
SPEC=rococo-local
# for the UI WS endpoint
HOST=$(firstword $(shell hostname -i))
run: run-parachain
run-parachain:
	$(MAKE) -s build chain=local
	# VLN devnet assets
	$(DOCKER) run --rm $(VLN) export-genesis-state \
		--parachain-id=2086 > $(BUILD)/local_vln_genesis_state
	$(DOCKER) run --rm $(VLN) export-genesis-wasm \
		--chain local > $(BUILD)/local_vln_genesis_wasm
	# Karura devnet assets
	$(DOCKER) run --rm $(KARURA) export-genesis-state \
		--chain karura-dev > $(BUILD)/karura-dev_genesis_state
	$(DOCKER) run --rm $(KARURA) export-genesis-wasm \
		--chain karura-dev > $(BUILD)/karura-dev_genesis_wasm

	$(DOCKER) run --rm $(POLKADOT) build-spec \
		--chain $(SPEC) --disable-default-bootnode --raw > $(BUILD)/relay-chain.json

	HOST=$(HOST) SPEC=$(SPEC) $(COMPOSE) -f devnet.yml up -d

stop-parachain:
	$(COMPOSE) -f devnet.yml down

# For simple needs run the single node standalone development chain
.PHONY: run-standalone
run-standalone: target/$(ENV)/$(BIN)_node
	$< --dev

.PHONY: clean-standalone
clean-standalone: target/$(ENV)/$(BIN)_node
	$< purge-chain --dev
