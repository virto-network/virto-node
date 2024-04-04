# Communities Pallet

This pallet enables people to form dynamic collectives refered to as
communities. In simpler terms, it can be considered a DAO Factory.

- [`Call`]
- [`Config`]

## Overview

The Communities pallet provides functionality for managing communities,
facilitating its participants to have governance over the community entity
(and its associated account) which can interect with other systems:

- Community registration and removal.
- Enrolling/removing members from a community.
- Promoting/demoting members within the community.
- Voting on proposals to enable community governance.

## Terminology

- **Community:** An entity comprised of _members_ —each one defined by their
  [`AccountId`][1]— with a given _description_ who can vote on _proposals_
  and actively take decisions on behalf of it. Communities are given a
  _treasury account_ they can use to hold assets.
- **Community Description:** A set of metadata used to identify a community
  distinctively. Typically, a name, a description and a URL.
- **Community Status:** A community can be either `active` or `blocked`.
- **Member:** An [`AccountId`][1] registered into the community as such. Can
  have a rank within it and vote in the community's polls.
- **Member Rank:** Members could have a rank within the community. This can
  determine a voting weight depending on the community's voting mechanism.
- **Proposal:** A poll that executes a [call][2] dispatch if approved when
  it's closed.
- **Community Account:** A keyless [`AccountId`][1] generated on behalf of
  the community. Like any regular account can hold balances. It can transfer
  funds via a privileged call executed by the community _admin_ or a call
  dispatched from a proposal.
- **Decision Method:** Can be either rank weighed, member-counted, or
  asset-weighed and determines how the votes of proposals will be tallied.

## Lifecycle

```ignore
[       ] --> [Pending]               --> [Active]            --> [Blocked]
create        set_metadata                set_metadata            unblock
                                          block                   
                                          add_member              
                                          remove_member
                                          promote
                                          demote
                                          set_voting_mechanism
```

## Implementations

> TODO: Define which traits we are defining/implementing.

## Interface

### Permissionless Functions

- [`apply_for`][c00]: Registers an appliation as a new community, taking an
  [existential deposit][3] used to create the community account.

### Permissioned Functions

Calling these functions requires being a member of the community.

- [`add_member`][c02]: Enroll an account as a community member. In theory,
  any community member should be able to add a member. However, this can be
  changed to ensure it is a privileged function.
- `vote`: Adds a vote into a community proposal.

### Privileged Functions

These functions can be called either by the community _admin_ or
dispatched through an approved proposal. !
- [`set_metadata`][c01]: Sets some [`CommunityMetadata`][t01] to describe
  the
community.
- [`remove_member`][c03]: Removes an account as a community member. While
  enrolling a member into the community can be an action taken by any
  member, the decision to remove a member should not be taken arbitrarily by
  any community member. Also, it shouldn't be possible to arbitrarily remove
  the community admin, as some privileged calls would be impossible execute
  thereafter.
- `promote`: Increases the rank of a member in the community.
- `demote`: Decreases the rank of a member in the community.
- `set_decision_method`: Means for a community to make decisions.

### Public Functions

- [`community`][g00]: Stores the basic information of the community. If a
  value exists for a specified [`ComumunityId`][t00], this means a community
  exists.
- [`metadata`][g01]: Stores the metadata regarding a community.

<!-- References -->
[1]: `frame_system::Config::AccountId`
[2]: https://docs.substrate.io/reference/glossary/#call
[3]: https://docs.substrate.io/reference/glossary/#existential-deposit

[t00]: `Config::CommunityId`
[t01]: `types::CommunityMetadata`
[c00]: `crate::Pallet::create`
[c01]: `crate::Pallet::set_metadata`
[c02]: `crate::Pallet::add_member`
[c03]: `crate::Pallet::remove_member`

[g00]: `crate::Pallet::community`
[g01]: `crate::Pallet::metadata`
[g02]: `crate::Pallet::membership`
[g03]: `crate::Pallet::members_count`
