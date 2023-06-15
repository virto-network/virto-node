# Virto Node ![Build][build_badge] ![Checks][checks_badge] [![PRs Welcome][pr_badge]](docs/CONTRIBUTING.adoc)

![Virto Logo](https://matrix.virto.community/_matrix/media/r0/download/virto.community/YGIstgrolxbgQAAsvhHdnZey)

[build_badge]: https://github.com/virto-network/virto-node/workflows/Blockchain%20Node/badge.svg
[checks_badge]: https://github.com/virto-network/virto-node/workflows/Checks/badge.svg
[pr_badge]: https://img.shields.io/badge/PRs-welcome-brightgreen.svg
[whitepaper]: https://virto.network/docs/whitepaper.html

[**Virto.Network**](https://virto.network), is the implementation of the [**Local Incentives Protocol**][whitepaper], 
a decentralized payments infrastructure for local communities with social impact.

Before Virto there is **Kreivo**, a token-less parachain for Kusama focused on real-world use cases such as
bridging Fiat money with decentralized on&off-ramps, d-commerce or any marketplace communities 
that transact with day-to-day life goods and services.

As part of our decentralized technology stack we tightly integrate with [Matrix](https://matrix.org), 
the decentralized communications protocol that we extend with our plugin runtime [Valor](https://github.com/virto-network/valor)
that allows the creation of high-level and convenient _decentralizable APIs_.

## Running the parachain

`virto-node` is a parachain, which means it needs to connect to a relay chain to finalize blocks. 
To ease the pain of setting up the different nodes required to test the network we use zombienet to set-up 
a test network to try out the different features of the parachain.

> You will need [`just`](https://just.systems/man/en/) and [Nushell](https://www.nushell.sh) 
to run our different build and test recipes. Check `just` for a list of available commands.

### Running local setup with zombienet

If you haven't already, run `just get-zombienet-dependencies` to get the `zombienet` binary 
plus the required `polkadot` and `polkadot-parachain`.

Then, `just zombienet`.

## Structure

### Core Pallets

- **Payments pallet** is the core payments primitive used by the network that enables secure reversible payments 
and configurable fees that allow for the _contributions_ to local communities.