# Virto Node ![Build](https://github.com/virto-network/virto-node/workflows/Blockchain%20Node/badge.svg) ![Checks](https://github.com/virto-network/virto-node/workflows/Checks/badge.svg) [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](docs/CONTRIBUTING.adoc)

This is the blockchain node of the **Virto.network**, a parachain set to become [Kusama](https://kusama.network)'s *"bridge to humanity"* thanks to the implementation of the People Local Interactions Protocol that seeks to maximize economic development of local communities.

Virto.network is the decentralized infastructure for **p2p marketplaces**, it tightly integrates with [Matrix](https://matrix.org), the decentralized communications protocol to allow communities around the world run any kind of local economic activity in a convenient way.

_Example of a p2p marketplace: fiat-crypto on-ramps_
![swap_flow](https://user-images.githubusercontent.com/1329925/119025875-db4d8d80-b9a4-11eb-9605-e7db6503156b.png)

To create more advanced interactions we use [Valor](https://github.com/virto-network/valor) the plugin runtime for _decentralizable APIs_.

## Running the parachain

`virto` is a parachain node, which means it _must_ connect to a relay chain and be onboarded in order to
produce blocks. To easy the pain of setting up the different nodes required to test the network we provide a 
[`devnet.yml`](devnet.yml)recipe for a multi-node local testnet that can be run with **`make run`** that also generates the 
required assets to on-board the network.

The devnet also spins up two Karura collators that can be onboarded to test cross-chain functionality.

💡 Note that nodes are launched in the background, to debug their output you can use podman/docker to follow the logs of any node in the multichain set-up(e.g. `podman logs -f devnet_virto_a`). Once you are done `make stop` will take care of removing the nodes.  

### Local development

In case you make changes to the runtime and what to test them in the parachain setup you can `make container` to 
build and containerize the node with your latest changes.(⚠️ at the moment the command will only work if your build environment is the same debian version as the [`Containerfile`](Containerfile)).

> To test xcm asset transfer check [vln-toolbox](https://github.com/virto-network/vln-toolbox)

## Project Structure

A Substrate project such as this consists of a number of components that are spread across a few
directories.

### Node

A blockchain node is an application that allows users to participate in a blockchain network.
Substrate-based blockchain nodes expose a number of capabilities:

-   Networking: Substrate nodes use the [`libp2p`](https://libp2p.io/) networking stack to allow the
    nodes in the network to communicate with one another.
-   Consensus: Blockchains must have a way to come to
    [consensus](https://substrate.dev/docs/en/knowledgebase/advanced/consensus) on the state of the
    network. Substrate makes it possible to supply custom consensus engines and also ships with
    several consensus mechanisms that have been built on top of
    [Web3 Foundation research](https://research.web3.foundation/en/latest/polkadot/NPoS/index.html).
-   RPC Server: A remote procedure call (RPC) server is used to interact with Substrate nodes.

There are several files in the `node` directory - take special note of the following:

-   [`chain_spec.rs`](./node/src/chain_spec.rs): A
    [chain specification](https://substrate.dev/docs/en/knowledgebase/integrate/chain-spec) is a
    source code file that defines a Substrate chain's initial (genesis) state. Chain specifications
    are useful for development and testing, and critical when architecting the launch of a
    production chain. Take note of the `development_config` and `testnet_genesis` functions, which
    are used to define the genesis state for the local development chain configuration. These
    functions identify some
    [well-known accounts](https://substrate.dev/docs/en/knowledgebase/integrate/subkey#well-known-keys)
    and use them to configure the blockchain's initial state.
-   [`service.rs`](./node/src/service.rs): This file defines the node implementation. Take note of
    the libraries that this file imports and the names of the functions it invokes. In particular,
    there are references to consensus-related topics, such as the
    [longest chain rule](https://substrate.dev/docs/en/knowledgebase/advanced/consensus#longest-chain-rule),
    the [Aura](https://substrate.dev/docs/en/knowledgebase/advanced/consensus#aura) block authoring
    mechanism and the
    [GRANDPA](https://substrate.dev/docs/en/knowledgebase/advanced/consensus#grandpa) finality
    gadget.

After the node has been [built](#build), refer to the embedded documentation to learn more about the
capabilities and configuration parameters that it exposes:

```shell
./target/release/virto_node --help
```

### Runtime

In Substrate, the terms
"[runtime](https://substrate.dev/docs/en/knowledgebase/getting-started/glossary#runtime)" and
"[state transition function](https://substrate.dev/docs/en/knowledgebase/getting-started/glossary#stf-state-transition-function)"
are analogous - they refer to the core logic of the blockchain that is responsible for validating
blocks and executing the state changes they define. The Substrate project in this repository uses
the [FRAME](https://substrate.dev/docs/en/knowledgebase/runtime/frame) framework to construct a
blockchain runtime. FRAME allows runtime developers to declare domain-specific logic in modules
called "pallets". At the heart of FRAME is a helpful
[macro language](https://substrate.dev/docs/en/knowledgebase/runtime/macros) that makes it easy to
create pallets and flexibly compose them to create blockchains that can address
[a variety of needs](https://www.substrate.io/substrate-users/).

Review the [FRAME runtime implementation](./runtime/src/lib.rs) included and note
the following:

-   This file configures several pallets to include in the runtime. Each pallet configuration is
    defined by a code block that begins with `impl $PALLET_NAME::Trait for Runtime`.
-   The pallets are composed into a single runtime by way of the
    [`construct_runtime!`](https://crates.parity.io/frame_support/macro.construct_runtime.html)
    macro, which is part of the core
    [FRAME Support](https://substrate.dev/docs/en/knowledgebase/runtime/frame#support-library)
    library.

### Pallets

The runtime in this project is constructed using many FRAME pallets that ship with the
[core Substrate repository](https://github.com/paritytech/substrate/tree/master/frame) and custom ones [defined in the `pallets`](./pallets/) directory.

A FRAME pallet is compromised of a number of blockchain primitives:

-   Storage: FRAME defines a rich set of powerful
    [storage abstractions](https://substrate.dev/docs/en/knowledgebase/runtime/storage) that makes
    it easy to use Substrate's efficient key-value database to manage the evolving state of a
    blockchain.
-   Dispatchables: FRAME pallets define special types of functions that can be invoked (dispatched)
    from outside of the runtime in order to update its state.
-   Events: Substrate uses [events](https://substrate.dev/docs/en/knowledgebase/runtime/events) to
    notify users of important changes in the runtime.
-   Errors: When a dispatchable fails, it returns an error.
-   Trait: The `Trait` configuration interface is used to define the types and parameters upon which
    a FRAME pallet depends.

## Polkadot-js UI

The [polkadot-js UI](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Ftestnet.valibre.dev#/explorer) UI can be used to interact with the virto-node. 
Some custom types will need to be specified in the `Settings > Developer` section of the UI. The definitions for these types 
can currently be found in the [types.json](types.json) file.
