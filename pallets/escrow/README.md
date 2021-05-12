# Escrow Pallet

This pallet will allow users to create an escrow onchain for a selected recipent. The pallet can be used to compliment the human swap pallet, to prove the existence of an escrow before the actual swap needs to be processed. The escrow pallet does not store the history of all the escrow transactions created by the user, it only cares about the current active escrow of the user/recipent.

## Terminology

- Created: An escrow has been created and amount reserved on chain.
- Released: The escrow amount has been released to the selected recipent

## Interface

#### Events

`EscrowCreated(T::AccountId, T::Asset, T::Amount)`,
`EscrowReleased(T::AccountId, EscrowId)`,
`EscrowCancelled(T::AccountId, T::AccountId)`

#### Extrinsics

`create(origin, recipent, currency_id, amount)` - Create an escrow for the given currencyid/amount
`release(origin, escrow_id)` - Release the escrow amount to recipent
`cancel(origin, escrow_id)` - Cancel the escrow and release the escrow amount to creator
`set_state(origin, from, recipent, new_state)` - Allows whitelisted judges to release/cancel an escrow

## Implementations

The RatesProvider module provides implementations for the following traits.
- [`EscrowHandler`](../../primitives/src/escrow.rs)

## GenesisConfig

The rates_provider pallet does not depend on the `GenesisConfig`

License: Apache-2.0