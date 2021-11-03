# Payments Pallet

This pallet allows users to create secure reversible payments that keep funds locked in a merchant's account until the off-chain goods are confirmed to be received.
Each payment gets assigned its own *judge* that can help resolve any disputes between the two parties. 

## Terminology

- Created: A payment has been created and the amount arrived to its destination but it's locked.
- Released: The payment amount has been released and became free balance in the recipient's account.

## Interface

#### Events

`PaymentCreated(T::AccountId, T::Asset, T::Amount)`,
`PaymentReleased(T::AccountId, PaymentId)`,
`PaymentCancelled(T::AccountId, T::AccountId)`

#### Extrinsics

`create(origin, recipient, currency_id, amount)` - Create an payment for the given currencyid/amount
`release(origin, payment_id)` - Release the payment amount to recipent
`cancel(origin, payment_id)` - Cancel the payment and release the payment amount to creator
`resolve(origin, from, recipient, new_state)` - Allows whitelisted judges to release/cancel an payment

## Implementations

The RatesProvider module provides implementations for the following traits.
- [`PaymentHandler`](../../primitives/src/payment.rs)

## GenesisConfig

The rates_provider pallet does not depend on the `GenesisConfig`

License: Apache-2.0
