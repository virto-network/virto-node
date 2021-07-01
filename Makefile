chain?=testnet
# out directory
BUILD=build
BIN=vln
SRC_DIRS=node runtime pallets primitives
SRC_FILES=$(shell find $(SRC_DIRS) -type f)

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
build: $(BUILD)/$(BIN) $(BUILD)/$(chain)_genesis_state \
	$(BUILD)/$(chain)_genesis_wasm \
	$(BUILD)/$(chain)_chainspec

# default bin is the parachain node
$(BUILD)/$(BIN): target/$(ENV)/$(BIN)_parachain
	@mkdir -p $(BUILD)
	@cp $< $@

$(BUILD)/$(chain)_genesis_state: $(BUILD)/$(BIN)
	$^ export-genesis-state --chain $(chain) > $@

$(BUILD)/$(chain)_genesis_wasm: $(BUILD)/$(BIN)
	$^ export-genesis-wasm --chain $(chain) > $@

$(BUILD)/$(chain)_chainspec: $(BUILD)/$(BIN)
	$^ build-spec --chain $(chain) --disable-default-bootnode > $@

$(TARGET): $(SRC_FILES)
	cargo build $(BUILD_FLAGS) -p $(BIN)-$(@:target/$(ENV)/$(BIN)_%=%)

# Containerize application. It uses the already built binary(e.g. during CI)
# and puts it in a cointainer, since the target image is a debian based container
# this won't likely work unless run in a similar debian installation.
.PHONY: container
DOCKER=podman
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
	cargo clippy -p $(BIN)-$(@:clippy_%=%)

.PHONY: run run-parachain
run: run-parachain

run-parachain: $(TARGET)
	./scripts/parachain-dev-setup.sh

.PHONY: dev
dev:
	cargo run -p vln-node -- --dev --tmp

