# Rates Provider Pallet

This pallet allows Liquidity providers to publish rates on chain for off-chain/on-chain transactions. The rates are determined based on the currency pair and the medium. The LPs dont publish the exact rates but rather the difference from the (official) oracle rates. 
Assume that COP-USDv price is 100, but a certain provider can process the payments at a rate that is 1% above the price-feed/oracle rate. In this case, the actual price of 100 will be stored by the price feed (and updated continously), but the margin of 1% or 100 basis points will be stored in the rates provider pallet.

## Terminology

- Provider: Any external user/company that provides liquidity to VLN.
- Medium: The route of this transaction, eg : BankTransfer, CashDeposit etc..
- Whitelist: Origin permitted to call the attestation extrinsic, must be a member of the whitelist membership pallet.

## Interface

#### Events

`RatesUpdated(T::AccountId, T::Asset, T::BaseAsset)`,
`RatesRemoved(T::AccountId, T::Asset, T::BaseAsset)`,

#### Extrinsics

`update_price(origin, base, quote, medium, rate)` - Create/update the rate that LP will accept
`remove_price(origin, base, quote, medium)` - Delist a pair/payment method for the LP

## Implementations

The RatesProvider module provides implementations for the following traits.
- [`RateProvider`](../../primitives/src/rates.rs)

## GenesisConfig

The rates_provider pallet does not depend on the `GenesisConfig`

License: Apache-2.0
