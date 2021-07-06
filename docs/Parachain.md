### How to run parachain

#### Build

Please note that we have two different runtimes
* `vln-node` - The standalone "validator" node runtime
* `vln-parachain` - The parachain-compliant "collator" node runtime which runs on Rococo

To build the `vln-parachain`.

```bash
make build
```

### Run

#### Local Testnet

Clone Polkadot (rococo-v1 branch):
```
cd polkadot/
cargo build --release
```

Run VLN parachain with rococo collators:
```
make run
```

To test xcm asset transfer clone and run [vln-toolbox](https://github.com/valibre-org/vln-toolbox)

