# Kreivo runtime [![Build](build_badge)](build_workflow) ![Checks][checks_badge] [![PRs Welcome][pr_badge]](docs/CONTRIBUTING.adoc)

![Virto Logo](https://matrix.virto.community/_matrix/media/r0/download/virto.community/YGIstgrolxbgQAAsvhHdnZey)

[build_badge]: https://github.com/virto-network/kreivo/actions/workflows/rust.yml/badge.svg
[build_workflow]: https://github.com/virto-network/kreivo/actions/workflows/rust.yml
[checks_badge]: https://github.com/virto-network/virto-node/workflows/Checks/badge.svg
[pr_badge]: https://img.shields.io/badge/PRs-welcome-brightgreen.svg
[whitepaper]: https://virto.network/docs/whitepaper.html

**Kreivo** is the implementation of the [**Local Incentives Protocol**][whitepaper], 
a decentralized infrastructure for real world buisinesses that contribute to local economic growth.

Kreivo is a token-less parachain in the [Kusama network](https://kusama.network/) focused on real-world use cases such as
bridging Fiat money with decentralized on&off-ramps, e-commerce or any kind of marketplaces 
that transact with day-to-day life goods and services.

As part of our decentralized technology stack we also integrate with [Matrix](https://matrix.org), 
the decentralized communications protocol that we extend with [VOS](https://github.com/virto-network/vos), 
a virual operating system to serve the off-chain companion of every organization part of the [Virto Network](https://virto.network).

## Running the parachain

The Kreivo runtime can be run with the `polkadot-parachain` node. 
To ease the pain of setting up the different nodes required to test the network we use zombienet to set-up 
a test network to try out the different features of the parachain.

> You will need [`just`](https://just.systems/man/en/) and [Nushell](https://www.nushell.sh) 
to run our different build and test recipes. Check `just` for a list of available commands.

### Running local setup with zombienet

If you haven't already, run `just get-zombienet-dependencies` to get the `zombienet` binary 
plus the required `polkadot` and `polkadot-parachain`.

Then, `just zombienet`.

## Structure

### Core modules

- **Payments pallet** is the core payments primitive used by the network that enables secure reversible payments
 and configurable fees that allow for the _contributions_ to local communities.
- **Communities pallet** integrates with Polkadot's *pallet referenda* to provide DAO functionality.
- [**Frame contrib**](https://github.com/virto-network/frame-contrib) includes **pallet pass** and
the traits that enable using nfts as memberships with a gas tank so users can transact without wallets or tokens.
