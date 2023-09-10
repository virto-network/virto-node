#![cfg_attr(not(feature = "std"), no_std)]

//! # Communities Pallet
//!
//! Part of the People Local Interactions Protocol, this pallet enables people
//! to unite and create local communities that share a common interest or
//! economic activity. In simpler terms, it can be considered a DAO Factory.
//!
//! - [`Call`]
//! - [`Config`]
//!
//! ## Overview
//!
//! The Communities pallet provides functionality for managing communities,
//! facilitating its participants to have governance over the community entity
//! (and its associated account), and running economic activities:
//!
//! - Community registration and removal.
//! - Validating a community challenge.
//! - Handling a community-governed treasury account.
//! - Enrolling/removing members from a community.
//! - Promoting/demoting members within the community.
//! - Running proposals to enable community governance.
//! - Issue governance tokens.
//! - Issue economic (sufficient) tokens.
//!
//! ## Terminology
//!
//! - **Community:** An entity comprised of _members_ —each one defined by their
//!   [`AccountId`][1]— with a given _description_ who can vote on _proposals_
//!   and actively take decisions on behalf of it. Communities are given a
//!   _treasury account_ and can issue _governance_ and _economic_ tokens. It is
//!   required that a community contributes to the network to be active and
//!   operate within it.
//! - **Community Description:** A set of metadata used to identify a community
//!   distinctively. Typically, a name, a list of locations (given as a list of
//!   one or more [`H3Index`][2]), and a list of URL links.
//! - **Community Status:** A community can be either `awaiting`, `active`, or
//!   `frozen` depending on whether the community has proven via a challenge
//!   it's actively contributing to the network with infrastructure provisioning
//!   (i.e. a [collator][3] node) or by depositing funds.
//! - **Validity Challenge:** A proof that a community is actively contributing
//!   to the network. The mechanisms for challenge verification are usually
//!   checked via an off-chain worker. Still, it's possible for a trusted origin
//!   to manually mark a community challenge as passed, effectively changing the
//!   status of the community to `active`.
//! - **Admin:** An [`AccountId`][1] registered into the community that is set
//!   as such. Can call [privileged functions](#privileged-functions) within the
//!   community.
//! - **Member:** An [`AccountId`][1] registered into the community as such. Can
//!   have a rank within it and vote in the community's polls.
//! - **Member Rank:** Members could have a rank within the community. This can
//!   determine a voting weight depending on the community's voting mechanism.
//! - **Proposal:** A poll with an optionally set duration that executes a
//!   [call][4] dispatch if approved when it's closed.
//! - **Treasury Account:** A keyless [`AccountId`][1] generated on behalf of
//!   the community. Can receive [payments][5], transfers, or payment [fees][6].
//!   It can transfer funds via a privileged call executed by the community
//!   _admin_ or a call dispatched from a proposal.
//! - **Governance Token:** A [non-sufficient fungible asset][7] issued and
//!   administered by the _Treasury Account_ of the community. Customarily, it's
//!   given among community members and can be used to vote depending on the
//!   voting mechanism set by the community.
//! - **Economic Token:** A [sufficient fungible asset][7] issued and
//!   administered by the _Treasury Account_ of the community. Generally used
//!   for monetary purposes, it can be transferred among members and non-members
//!   of the community, used to pay network fees, and for [payments][5] and its
//!   corresponding [fees][6].
//!
//! ## Goals
//!
//! The _"communities"_ are designed to facilitate the following use cases:
//!
//! - Enable entities (i.e. DAOs) or local-bound groups of people (physical
//!   communities) that share common interests to create markets.
//! - Allow _communities_ can receive taxes (as in [payment fees][5]) and be
//!   self-sustainable.
//! - Let such _communities_ to sovereignly decide how to spend those gathered
//!   funds by running and voting on proposals.
//!
//! ## Lifecycle
//!
//! ```ignore
//! [       ] --> [Awaiting]              --> [Active]            --> [Frozen]
//! apply         set_metadata                set_metadata            set_metadata
//!               fulfill_challenge           add_member              thaw
//!               force_complete_challenge    remove_member
//!                                           promote_member
//!                                           demote_member
//!                                           issue_token
//!                                           open_proposal
//!                                           vote_proposal
//!                                           close_proposal
//!                                           assets_transfer
//!                                           balance_transfer
//!                                           set_admin
//!                                           set_voting_mechanism
//!                                           set_sufficient_asset
//!                                           freeze
//! ```
//!
//! ## Implementations
//!
//! > TODO: Define which traits we are defining/implementing.
//!
//! ## Interface
//!
//! ### Permissionless Functions
//!
//! - [`apply`][8]: Registers an appliation as a new community, taking an
//!   [existential deposit][9] used to create the community account.
//!
//! ### Permissioned Functions
//!
//! Calling these functions requires being a member of the community.
//!
//! - `fulfill_challenge`: Submit the challenge proof to validate the
//!   contribution status of the community.
//! - `add_member`: Enroll an account as a community member. In theory, any
//!   community member should be able to add a member. However, this can be
//!   changed to ensure it is a privileged function.
//! - `open_proposal`: Creates a proposal to be voted by the community. At
//! this point, there can only be a single proposal at a time.
//! - `vote_proposal`: Adds a vote into a community proposal.
//!
//! ### Privileged Functions
//!
//! These functions can be called either by the community _admin_ or
//! dispatched through an approved proposal. !
//! - `set_metadata`: Sets some [`CommunityMetadata`][10] to describe the
//! community.
//! - `remove_member`: Removes an account as a community member. While enrolling
//!   a member into the community can be an action taken by any member, the
//!   decision to remove a member should not be taken arbitrarily by any
//!   community member.
//! - `promote_member`: Increases the rank of a member in the community. ! -
//!   `demote_member`: Decreases the rank of a member in the community.
//! - `issue_token`: Creates a token that is either governance (only one per
//!   community allowed) or economic. While the first economic token is
//!   _"free"_," further ones would be subject to network-wide referenda.
//! - `close_proposal`: Forcefully closes a proposal, dispatching the call when
//!   approved.
//! - `assets_transfer`: Transfers an amount of a given asset from the treasury
//!   account to a beneficiary.
//! - `balance_transfer`: Transfers funds from the treasury account to a
//!   beneficiary.
//! - `set_admin`: Sets an [`AccountId`][1] of the _admin_ of the community.
//!   Ensures that the specified account is a member of the community.
//! - `set_voting_mechanism`: Transfers funds from the treasury account to a
//!   beneficiary.
//!
//! ### Root Functions
//!
//! - `force_complete_challenge`: Marks a challenge as passed. This can lead to
//!   the activation of a community if all challenges are passed.
//! - `force_increase_economic_token_limit`: Increases the amount of economic
//!   tokens a community can issue.
//!
//! ### Public Functions
//!
//! [1]: `frame_system::Config::AccountId`
//! [2]: https://h3geo.org/docs/highlights/indexing
//! [3]: https://docs.substrate.io/reference/glossary/#collator
//! [4]: https://docs.substrate.io/reference/glossary/#call
//! [5]: https://github.com/virto-network/virto-node/tree/master/pallets/payments
//! [6]: https://github.com/virto-network/virto-node/pull/282
//! [7]: https://paritytech.github.io/substrate/master/pallet_assets/index.html#terminology
//! [8]: `crate::Pallet::apply`
//! [9]: https://docs.substrate.io/reference/glossary/#existential-deposit
//! [10]: `types::CommunityMetadata`
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod functions;

pub mod types;
pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		traits::tokens::{fungible, fungibles},
		Parameter,
	};
	use frame_system::pallet_prelude::*;
	use types::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it
	/// depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// This type represents an unique ID for the community
		type CommunityId: Parameter + MaxEncodedLen;

		/// This type represents a rank for a member in a community
		type MemberRank: Default + Parameter + MaxEncodedLen;

		/// Type represents interactions between fungibles (i.e. assets)
		type Assets: fungibles::Inspect<Self::AccountId>
			+ fungibles::Mutate<Self::AccountId>
			+ fungibles::Create<Self::AccountId>
			+ fungibles::Destroy<Self::AccountId>;

		/// Type represents interactions between fungibles (i.e. assets)
		type Balances: fungible::Inspect<Self::AccountId>
			+ fungible::Mutate<Self::AccountId>
			+ fungible::InspectFreeze<Self::AccountId>
			+ fungible::MutateFreeze<Self::AccountId>;

		/// Because this pallet emits events, it depends on the runtime's
		/// definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;

		/// The Communities' pallet id, used for deriving its sovereign account
		/// ID.
		#[pallet::constant]
		type PalletId: Get<frame_support::PalletId>;

		/// The Communities' freeze identifier, used for identifying freezes
		/// created by the pallet.
		#[pallet::constant]
		type FreezeIdentifier: Get<<Self::Balances as fungible::InspectFreeze<Self::AccountId>>::Id>;
	}

	/// Store the basic information of the community. If a value exists for a
	/// specified [`ComumunityId`], this means a community exists.
	#[pallet::storage]
	#[pallet::getter(fn community)]
	pub(super) type CommunityInfo<T> = StorageMap<_, Blake2_128Concat, CommunityIdOf<T>, Community<T>>;

	/// Store the list of community members. If some values exist under a
	/// specified [`ComumunityId`] prefix, this means a community exists.
	#[pallet::storage]
	#[pallet::getter(fn member_rank_for)]
	pub(super) type CommunityMembers<T> =
		StorageDoubleMap<_, Blake2_128Concat, CommunityIdOf<T>, Blake2_128Concat, AccountIdOf<T>, MemberRankOf<T>>;

	/// Store the count of community members. This simplifies the process of
	/// keeping track of members' count.
	#[pallet::storage]
	#[pallet::getter(fn members_count)]
	pub(super) type CommunityMembersCount<T> = StorageMap<_, Blake2_128Concat, CommunityIdOf<T>, u128>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A [`Commmunity`][`types::Community`] has been created.
		CommunityCreated { id: T::CommunityId, who: T::AccountId },
	}

	// Errors inform users that something worked or went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The community doesn't exist in storage, nor they have members.
		CommunityDoesNotExist,
		/// A community with the same [`CommunityId`][`Config::CommunityId`]
		/// already exists, therefore cannot be applied for.
		CommunityAlreadyExists,
		/// The specified [`AccountId`][`frame_system::Config::AccountId`] is
		/// not a member of the community
		NotAMember,
		/// The specified [`AccountId`][`frame_system::Config::AccountId`] is
		/// already a member of the community
		AlreadyAMember,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke
	// state changes. These functions materialize as "extrinsics", which are often
	// compared to transactions. Dispatchable functions must be annotated with a
	// weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Registers an appliation as a new community, taking an
		/// [existential deposit][9] used to create the community account.
		///
		/// [9]: https://docs.substrate.io/reference/glossary/#existential-deposit
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::apply())]
		pub fn apply(origin: OriginFor<T>, community_id: T::CommunityId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_register_community(&who, &community_id)?;
			Self::do_create_community_account(&who, &community_id)?;

			// Emit an event.
			Self::deposit_event(Event::CommunityCreated { id: community_id, who });

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}
}
