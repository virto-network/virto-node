# Communities Pallet

Part of the People Local Interactions Protocol, this pallet enables people
to unite and create local communities that share a common interest or
economic activity. In simpler terms, it can be considered a DAO Factory.

## Overview

The Communities pallet provides functionality for managing communities,
facilitating its participants to have governance over the community entity
(and its associated account), and running economic activities:

- Community registration and removal.
- Validating a community challenge.
- Handling a community-governed treasury account.
- Enrolling/removing members from a community.
- Promoting/demoting members within the community.
- Running proposals to enable community governance.
- Issue governance tokens.
- Issue economic (sufficient) tokens.

## Terminology

- **Community:** An entity comprised of _members_ —each one defined by their
  [`AccountId`][1]— with a given _description_ who can vote on _proposals_
  and actively take decisions on behalf of it. Communities are given a
  _treasury account_ and can issue tokens. It is required that a community
  contributes to the network to be active and operate within it.
- **Community Description:** A set of metadata used to identify a community
  distinctively. Typically, a name, a list of locations (given as a list of
  one or more [`H3Index`][2]), and a list of URL links.
- **Community Status:** A community can be either `awaiting`, `active`, or
  `frozen` depending on whether the community has proven via a challenge
  it's actively contributing to the network with infrastructure provisioning
  (i.e. a [collator][3] node) or by depositing funds.
- **Validity Challenge:** A proof that a community is actively contributing
  to the network. The mechanisms for challenge verification are usually
  checked via an off-chain worker. Still, it's possible for a trusted origin
  to manually mark a community challenge as passed, effectively changing the
  status of the community to `active`.
- **Admin:** An [`AccountId`][1] registered into the community that is set
  as such. Can call [privileged functions](#privileged-functions) within the
  community.
- **Member:** An [`AccountId`][1] registered into the community as such. Can
  have a rank within it and vote in the community's polls.
- **Member Rank:** Members could have a rank within the community. This can
  determine a voting weight depending on the community's voting mechanism.
- **Proposal:** A poll with an optionally set duration that executes a
  [call][4] dispatch if approved when it's closed.
- **Treasury Account:** A keyless [`AccountId`][1] generated on behalf of
  the community. Can receive [payments][5], transfers, or payment [fees][6].
  It can transfer funds via a privileged call executed by the community
  _admin_ or a call dispatched from a proposal.
- **Community Token:** A Community might create and manage tokens by
  dispatching the respective call from the corresponding origin. These tokens
  also might be used to vote if set via Voting Mechanism.
- **Voting Method:** Can be either rank weighed, member-counted, or asset-weighed
  and determines how the votes of proposals will be tallied.

## Goals

The _"communities"_ are designed to facilitate the following use cases:

- Enable entities (i.e. DAOs) or local-bound groups of people (physical
  communities) that share common interests to create markets.
- Allow _communities_ can receive taxes (as in [payment fees][5]) and be
  self-sustainable.
- Let such _communities_ to sovereignly decide how to spend those gathered
  funds by running and voting on proposals.

## Lifecycle

```ignore
[       ] --> [Awaiting]              --> [Active]            --> [Frozen]      --> [Blocked]
apply         set_metadata                set_metadata            set_metadata      unblock
              fulfill_challenge           block                   block
              force_complete_challenge    add_member              thaw
                                          remove_member
                                          promote_member
                                          demote_member
                                          issue_token
                                          open_proposal
                                          vote_proposal
                                          close_proposal
                                          assets_transfer
                                          balance_transfer
                                          set_admin
                                          set_voting_mechanism
                                          set_sufficient_asset
                                          freeze
```

## Implementations

> TODO: Define which traits we are defining/implementing.

## Interface

### Permissionless Functions

- `apply`: Registers an appliation as a new community, taking an
  [existential deposit][8] used to create the community account.

### Permissioned Functions

Calling these functions requires being a member of the community.

- `fulfill_challenge`: Submit the challenge proof to validate the
  contribution status of the community.
- `add_member`: Enroll an account as a community member. In theory,
  any community member should be able to add a member. However, this can be
  changed to ensure it is a privileged function.
- `open_proposal`: Creates a proposal to be voted by the community. At
  this point, there can only be a single proposal at a time.
- `vote_proposal`: Adds a vote into a community proposal.

### Privileged Functions

These functions can be called either by the community _admin_ or
dispatched through an approved proposal. !

- `set_metadata`: Sets some [`CommunityMetadata`][t01] to describe
  the community.
- `remove_member`: Removes an account as a community member. While
  enrolling a member into the community can be an action taken by any
  member, the decision to remove a member should not be taken arbitrarily by
  any community member. Also, it shouldn't be possible to arbitrarily remove
  the community admin, as some privileged calls would be impossible execute
  thereafter.
- `promote_member`: Increases the rank of a member in the community. ! -
  `demote_member`: Decreases the rank of a member in the community.
- `issue_token`: Creates a token that is either governance (only one per
  community allowed) or economic. While the first economic token is
  _"free"_," further ones would be subject to network-wide referenda.
- `close_proposal`: Forcefully closes a proposal, dispatching the call when
  approved.
- `assets_transfer`: Transfers an amount of a given asset from the treasury
  account to a beneficiary.
- `balance_transfer`: Transfers funds from the treasury account to a
  beneficiary.
- `set_sufficient_asset`: Marks an [asset][7] issued by the community as
  sufficient. Only one asset at a time can be marked as such.
- `set_admin`: Sets an [`AccountId`][1] of the _admin_ of the community.
  Ensures that the specified account is a member of the community.
- `set_voting_mechanism`: Transfers funds from the treasury account to a
  beneficiary.

### Root Functions

- `force_complete_challenge`: Marks a challenge as passed. This can lead to
  the activation of a community if all challenges are passed.
- `block`: Sets a community status as `blocked`, specifiying a reason for the blocking.
- `unblock`: Sets a community status back to the previously given state.

### Public Functions

- `community`: Stores the basic information of the community. If a value exists for a
  specified [`ComumunityId`][t00], this means a community exists.
- `asset_id`: Stores the ID of the [asset][7] for a given [`CommunityId`][t00].
- `metadata`: Stores the metadata regarding a community.
- `membership`: Stores the information of a community (specified by its
  [`CommunityId`][t00]) member (specified by it's [`AccountId`][1]).
- `members_count`: Store the count of community members. This simplifies the process of keeping track of members' count.

<!-- References -->

[1]: https://paritytech.github.io/substrate/master/frame_system/pallet/trait.Config.html#associatedtype.AccountId
[2]: https://h3geo.org/docs/highlights/indexing
[3]: https://docs.substrate.io/reference/glossary/#collator
[4]: https://docs.substrate.io/reference/glossary/#call
[5]: https://github.com/virto-network/virto-node/tree/master/pallets/payments
[6]: https://github.com/virto-network/virto-node/pull/282
[7]: https://paritytech.github.io/substrate/master/pallet_assets/index.html#terminology
[8]: https://docs.substrate.io/reference/glossary/#existential-deposit
[t00]: src/lib.rs#L237
[t01]: src/types.rs#L57
