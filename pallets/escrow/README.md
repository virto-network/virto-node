# Payment Pallet

This pallet will allow users to create an payment onchain for a selected recipent. The pallet can be used to compliment the human swap pallet, to prove the existence of an payment before the actual swap needs to be processed. The payment pallet does not store the history of all the payment transactions created by the user, it only cares about the current active payment of the user/recipent.

## Terminology

- Created: An payment has been created and amount reserved on chain.
- Released: The payment amount has been released to the selected recipent

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