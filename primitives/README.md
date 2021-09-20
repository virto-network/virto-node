# Virto Network - Primitives

This module defines the basic primitives that form the basis of Virto Network.

## Asset

An asset is meant to represent the synthetic version of any fiat/crypto currency that is allowed onchain.

```rust
pub enum Asset {
    Collateral(Collateral),
    Fiat(Fiat),
    Network(NetworkAsset),
    Usdv,
}
```

#### Collateral

Collateral lists all the cryptocurrencies that can be deposited to create assets onchain.

```rust
pub enum Collateral {
        Usdc = "USDC",
    }
```

#### Fiat

Fiat lists all the fiat currencies that can be deposited to create assets onchain.

```rust
pub enum Fiat {
        Cop = "COP",
        Vez = "VEZ",
    }
```

#### Network

Network lists all the currencies that we currently support for xcm transfer from other parachains and relay chain.

```rust
pub enum NetworkAsset {
        ACA = "ACA",
        AUSD = "AUSD",
        DOT = "DOT",
    }
```

