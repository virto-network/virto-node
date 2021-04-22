# Rates Provider Pallet

This pallet allows Liquidity providers to publish rates on chain for off-chain/on-chain transactions. The rates are determined based on the currency pair and the payment method. The LPs dont publish the exact rates but rather the difference from the (official) oracle rates. 
Assume that COP-USDv price is 100, but a certain LP can process the payments at a rate that is 1% above the oracle rate. In this case, the actual price of 100 will be stored by the oracle pallet (and updated continously), but the margin of 1% or 100 basis points will be stored in the rates provider pallet.

## Terminology

- Liquidity Provider: Any external user/company that provides liquidity to VLN.
- Whitelist: Origin permitted to call the attestation extrinsic, must be a member of the whitelist membership pallet.

## Interface

#### Events

`RatesUpdated(T::AccountId, T::CurrencyId, T::CurrencyId)`,
`RatesRemoved(T::AccountId, T::CurrencyId, T::CurrencyId)`,

#### Extrinsics

`update_price(origin, from_currency, to_currency, payment_method, rate)` - Create/update the rate that LP will accept
`remove_price(origin, from_currency, to_currency, payment_method)` - Delist a pair/payment method for the LP

## GenesisConfig

The rates_provider pallet does not depend on the `GenesisConfig`

License: Apache-2.0
