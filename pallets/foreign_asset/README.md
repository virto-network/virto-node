# Foreign Asset Pallet

Foreign assets are any off-chain crypto or fiat asset that an attester claims to have. Foreign assets can be minted on chain in a permissioned way if the attester is trusted (in a whitelist) or in a permissionless way (not supported yet) by locking some funds as safeguard.

## Terminology

- Attestor: Any external user/company that provides liquidity to VLN. The permission to be an attestor will initially be restricted via a whitelist, but will be opened up in future to all users.
- [Asset](../../primitives/README.md##Asset): Onchain record of a crypto/fiat asset that the attesttor claims to posses. This asset is represented using the [tokens-pallet](https://github.com/stanly-johnson/open-runtime-module-library/tree/master/tokens) and is locked upon minting (cannot be transferred onchain).
- Mint: The process of creating an asset on VLN by an attestor claim.
- Whitelist: Origin permitted to call the attestation extrinsic, must be a member of the whitelist membership pallet.

## Interface

#### Events

`Attestation(AccountId, CurrencyId)` - Denotes a new asset has been minted

#### Extrinsics

`attest(origin,currency,amount)` - Create a new asset on chain of type `currency` and `amount`



## GenesisConfig

The foreign_asset pallet does not depend on the `GenesisConfig`


License: Apache-2.0
