### How to run parachain

#### Build

Please note that we have two different runtimes
* `vln-node` - The standalone "validator" node runtime
* `vln-parachain` - The parachain-compliant "collator" node runtime which runs on Rococo

For development, we recommend to run the `vln-node`.

```bash
cargo build --release -p vln-parachain
```

### Run

#### Local Testnet

Clone Polkadot (rococo-v1 branch):
```
cd polkadot/
cargo build --release --features real-overseer

./target/release/polkadot build-spec --chain rococo-local --raw --disable-default-bootnode > rococo_local.json

./target/release/polkadot --chain ./rococo_local.json -d cumulus_relay1 --validator --bob --port 50555
./target/release/polkadot --chain ./rococo_local.json -d cumulus_relay0 --validator --alice --port 50556
```

Run VLN parachain:
```
./target/release/vln_parachain -d local-test --collator --alice --ws-port 9945 --parachain-id 200 -- --chain ../polkadot/rococo_local.json
```
