# Payments Pallet

This pallet allows users to create secure reversible payments that keep funds locked in a merchant's account until the off-chain goods are confirmed to be received.
Each payment gets assigned its own *judge* that can help resolve any disputes between the two parties. 


## Terminology

- Created: A payment has been created and the amount arrived to its destination but it's locked.
- NeedsReview: The payment has bee disputed and is awaiting settlement by a judge.
- IncentivePercentage: A small share of the payment amount is held in escrow until a payment is completed/cancelled. The Incentive Percentage represents this value.
- Resolver Account: A resolver account is assigned to every payment created, this account has the privilege to cancel/release a payment that has been disputed.
- Remark: The pallet allows to create payments by optionally providing some extra(limited) amount of bytes, this is reffered to as Remark. This can be used by a marketplace to seperate/tag payments.

## Interface

#### Events

`PaymentCreated(T::AccountId, T::Asset, T::Amount)`,
`PaymentReleased(T::AccountId, PaymentId)`,
`PaymentCancelled(T::AccountId, T::AccountId)`

#### Extrinsics

`create(origin, recipient, currency_id, amount)` - Create an payment for the given currencyid/amount
`release(origin, payment_id)` - Release the payment amount to recipent
`cancel(origin, payment_id)` - Cancel the payment and release the payment amount to creator
`resolve_release_payment(origin, from, recipient)` - Allows assigned judge to release a payment
`resolve_cancel_payment(origin, from, recipient)` - Allows assigned judge to cancel a payment

## Implementations

The RatesProvider module provides implementations for the following traits.
- [`PaymentHandler`](../../primitives/src/payment.rs)

## GenesisConfig

The rates_provider pallet does not depend on the `GenesisConfig`

License: Apache-2.0
