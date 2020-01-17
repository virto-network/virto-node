Valibre aims to be an open source set of technologies that allows [Valiu](https://www.valiu.com) become THE
decentralized remittances network.

# A remittances network

Remittances are a common way of money transfer specially popular with foreign workers sending money back to their country of origin. It has grown to be a huge market on the hundreds of billions of dollars annually with great impact in the economy of developing countries and the life of many of its citizens.

It all starts with _Alice_ daughter of _Bob_, she needs to send some of her well earned fiat money back home to help the family's economy.

![Bob to Alice](https://gateway.ipfs.io/ipfs/QmRZEnVqCGBiWs1hNSH4P5fcNDrYqrMd4HRJ5U3eSBcR8r)

To do so Alice has some options, she could make an international bank transfer which is often slow and pricey or use a multinational company that is also expensive due to their operational costs and monopoly in the market. Without thinking twice she chooses a blockchain based solution that will safe her money and time, every penny and second counts when Bob is in a bad economic situation and his country suffers from hyperinflation.

## Sending crypto money

**First scenario.** The easiest thing of all is when Alice gets her money in the form of a digital stable coin and Bob can spend those coins directly or hold them knowing he won't be losing money thanks to the coin's stable price.

> 1. Alice creates a transfer of 10 vUSD(a stable coin pegged to the USD) to Bob's address from her digital wallet.
> 2. Bob immediately receives the 10 vUSD with virtually no fee and spends it in the local super market that accepts this crypto currency.

**Second scenario.** Alice sends her virtualized local currency and Bob receives it automatically converted to his own local virtual currency.

> 1. Alice creates a transfer of 1000 vCOP(virtual colombian peso) to Bob.
> 2. Bob is known to prefer vVES(virtual sovereign bolivar)
> 3. The network uses trusted external sources of information(oracles) to know how much vCOP and vVES are worth against the vUSD and update an internal table of exchange rates.
> 4. The network converts vCOP to vVES knowing how vCOP>vUSD and vUSD>vVES convert and sends money to Bob.
> 5. Bob is happy he got his 10100 vVES so quick.

**Third scenario.** Alice sends an arbitrary crypto currency to Bob who expects another one.

> 1. Alice creates a transfer of 10 VAL(some crypto currency traded in the open market)
> 2. Bob still likes to receive vUSD.
> 3. The network uses a decentralized exchange(DEX) to swap VAL for vUSD and send to Bob.
> 4. Bob buys a burrito for 1 vUSD.

**Fourth scenario.** Alice and Bob say crypto whattt? They can only make local bank transfers or deposit cash in a physical location.
_Charlie_ to the rescue! he's a crypto guru with money all over the place, he and many other people help providing liquidity to the network to make fiat remittances possible in exchange of the nice feeling of helping others and yeah an economic incentive as well.

![Charly and pool](https://gateway.ipfs.io/ipfs/QmWpXTAE9rTDu1gCFustSbgjtaB86MetyG55RwCgHQXQWT)

> 1. Charlie locks a lot of his vUSD in the crypto pool.
> 2. Charlie makes a claim to the virtual fiat pool that he has an account full of COP in Colombia.
> 3. Charlie makes the same claim about having tons of VES in Venezuela, if he was a newbie in the network he would probably not be able to make such a big claim.

> 4. Alice wants to virtualize 3000 COP creating a deposit.
> 5. The network checks the crypto pool to confirm there are enough funds that will be coming from Charlie.
> 6. Alice confirms and the network puts 1 vUSD from Charlie in a escrow account.
> 7. Alice receives an encrypted chat message from the network's bot with some bank details(Charlie's).
> 8. Alice makes a bank transfer to Charlie who is notified of the transfer.
> 9. Charlie confirms the funds in his account and releases the locked crypto in the escrow account.
> 10. The network makes the conversion of vUSD to vCOP and now Alice has 3000 vCOP.

> 11. Alice creates a transfer of 2000 vCOP to Bob.
> 12. Bob receives 20200 vVES after conversion by the network.

> 13. Bob wants to cash out his 20200 vVES creating a withdrawal.
> 14. The network checks the virtual fiat pool to confirm there are funds available.
> 15. The network converts and locks Bob's vVES as vUSD in a escrow account.
> 16. Charlie is notified he needs to transfer 20200 VES to Bob's account.
> 17. Bob confirms he got the money and vUSD is released from the escrow.
> 18. Alice, Bob and Charlie are very happy.

**Bonus scenario** Uncle _Dave_ is the fancy family member in Europe that sends money to Bob from time to time. Of course Charlie also has money in Europe and both of them use a cool bank(like [bunq](https://www.bunq.com)) that provides an API for the network developers to integrate with.

![Fancy Dave](https://gateway.ipfs.io/ipfs/QmbyMFcZ8ZsBkrnUpjYCkzkB3dujP8iM16LzxWyo7qZM5W)

> 1. Dave wakes up feeling generous and asks his AI assistant, "please send 100000 VES to Bob".
> 2. The assistant initiates the transfer using a standard protocol(e.g. `web+wallet://tx/VES?to=Bob&a=100000`) which the open source web wallet with awesome user experience automatically picks since it had previously registered itself to handle the protocol.
> 3. The wallet is a [PWA](https://developers.google.com/web/progressive-web-apps) that includes a [WebAssembly](https://webassembly.org) compiled blockchain light client built with [Substrate](https://substrate.dev) part of the [Polkadot network](https://polkadot.network) distributed over [IPFS](https://ipfs.io) as a normal page accessible from a browser on [https://val.app](https:://val.app) or as a [Web Bundle](https://web.dev/web-bundles) in case internet access is hard or heavily censored and to ensure maximum portability. The wallet then uses [WebAuthn](https://webauthn.io) APIs to unlock Dave's private keys stored in his hardware wallet connected over Web Bluetooth or Web USB by only asking for his fingerprint confirmation.
> 4. Dave's bank shows a notification with a confirmation of the transfer to Charlie who doesn't do anything for now since his vUSD was already locked and released as vVES to Bob once his wallet checked for him he got the money in his bank account.
> 5. Now Charlie and Bob follow the standard procedure to cash out the 100000 VES hoping banks in his region become more open providing APIs to integrate with or looking forward that close future when their digital money is all they need and nobody ever mentions cash and banks anymore. 🙊

Phew! That's enough for now 😅 This is our happy path, you can see it from the happy faces of _Alice_, _Bob_, _Charlie_ and _Dave_ who are moving money around in a secure, cheap, fast and convenient way. But don't think we are done just yet! there's plenty of details to iron out to make the network a happy place for all because Devil is in the details so we'll have to deep dive into how we can make it hard for the unfriendly Dev to ruin everyone's happiness and how to reward the brave ones that keep him at bay.

## A remittances parachain

_TODO_: Para what? Patience We'll get there ...

### POC

Based on the above scenarios, **scenario one** is the quickest way to get started and get ourselves familiar with the substrate framework, _Alice_ and _Bob_ will run their nodes as a cli application, they will have some vUSD available from the beginning that they will trade using the [polkadot.js app](https://polkadot.js.org/apps). Right after that simple milestone add **scenario two** to the mix will the next natural step along with a minimal reference web UI that enables both parties make the transfer in a simpler way.


# Development

## Build

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Initialize your Wasm Build environment:

```bash
./scripts/init.sh
```

Build Wasm and native code:

```bash
cargo build --release
```

## Run

### Single node development chain

Purge any existing developer chain state:

```bash
./target/release/node purge-chain --dev
```

Start a development chain with:

```bash
./target/release/node --dev
```

Detailed logs may be shown by running the node with the following environment variables set: `RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev`.

### Multi-node local testnet

If you want to see the multi-node consensus algorithm in action locally, then you can create a local testnet with two validator nodes for Alice and Bob, who are the initial authorities of the genesis chain that have been endowed with testnet units.

Optionally, give each node a name and expose them so they are listed on the Polkadot [telemetry site](https://telemetry.polkadot.io/#/Local%20Testnet).

You'll need two terminal windows open.

We'll start Alice's substrate node first on default TCP port 30333 with her chain database stored locally at `/tmp/alice`. The bootnode ID of her node is `QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR`, which is generated from the `--node-key` value that we specify below:

```bash
cargo run -- \
  --base-path /tmp/alice \
  --chain=local \
  --alice \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --validator
```

In the second terminal, we'll start Bob's substrate node on a different TCP port of 30334, and with his chain database stored locally at `/tmp/bob`. We'll specify a value for the `--bootnodes` option that will connect his node to Alice's bootnode ID on TCP port 30333:

```bash
cargo run -- \
  --base-path /tmp/bob \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR \
  --chain=local \
  --bob \
  --port 30334 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --validator
```

Additional CLI usage options are available and may be shown by running `cargo run -- --help`.
