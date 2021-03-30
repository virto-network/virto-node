# Backed Asset Pallet

This pallet allows creation of asset that is backed by one or more similar assets keeping track of the share that each collateral holds. Any liquidity provider can create a backed asset by depositing collateral to this pallet.

## Terminology

- Liquidity Provider : Any external user/company that provides liquidity to VLN by depositing up permitted collateral to mint assets on VLN.
- [Asset](../../primitives/README.md##Asset): Onchain record of a crypto asset that is issued relative to the collateral locked up by the caller.

## Interface
### Implementations

The backed asset module provides implementations for the following traits.

- [`Currency`](https://docs.rs/frame-support/latest/frame_support/traits/trait.Currency.html): Functions for dealing with a
fungible assets system.
#### Events

`Mint(AccountId, CurrencyId)` - Denotes a new asset has been minted

#### Extrinsics

`mint(origin,collateral,amount)` - Use some valid collateral to create the same amount of backed-assets updating the share ratio of the collateral compared to other collaterals backing the same asset.

## GenesisConfig

The backed asset pallet does not depend on the `GenesisConfig`


License: Apache-2.0
