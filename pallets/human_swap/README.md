# Human Swap Pallet

For swaps that require human interaction due to one of the trading pairs being a foreign asset(an asset that exists off-chain).  
Users of this pallet specify a human they want to trade with based on the convenience of their rates, the party sending cryptocurrency
gets its funds locked in a scrow and the other receives the information necesary to conduct the off-chain payment.  
It's worth noting that no user information is stored on-chain but only a hash of a human/machine readable document stored elsewhere(e.g. IPFS) 
that both parties have access to(same for proofs). If the storage is public, clients are encouraged to encrypt the contents of 
the document with the other party's public key.

![swap_flow](https://user-images.githubusercontent.com/1329925/115689673-99f1a000-a35c-11eb-9678-4a2ac137b755.png)

This pallet tracks the state of the swaps which are updated based on actions form the user and the human involved. In the image each of the actions for either a `swap_in` or a `swap_out` changes the state of the overal swap which looks something like
```rust
struct Swap {
  human: AccountId,
  type: SwapType,
  rate: PairRate,
  amount: Balance,
}

enum SwapType {
  In {
    Created,
    Accepted(Cid),
    Rejected(Reason),
    Confirmed(Proof),
    Expired,
    Completed,
  },
  Out {
    Created(Cid),
    Accepted,
    Rejected(Reason),
    Confirmed(Proof),
    Expired,
    Completed,
  },
}
```
