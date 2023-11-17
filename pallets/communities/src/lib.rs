#![cfg_attr(not(feature = "std"), no_std)]
#![feature(let_chains)]
#![feature(trait_alias)]

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
//! (and its associated account) which can interect with other systems:
//!
//! - Community registration and removal.
//! - Validating a community challenge.
//! - Enrolling/removing members from a community.
//! - Promoting/demoting members within the community.
//! - Voting on proposals to enable community governance.
//!
//! ## Terminology
//!
//! - **Community:** An entity comprised of _members_ —each one defined by their
//!   [`AccountId`][1]— with a given _description_ who can vote on _proposals_
//!   and actively take decisions on behalf of it. Communities are given a
//!   _treasury account_ and can issue tokens. Communities can be challenged to
//!   decide if they should stay active.
//! - **Community Description:** A set of metadata used to identify a community
//!   distinctively. Typically, a name, a description and a URL.
//! - **Community Status:** A community can be either `awaiting`, `active`, or
//!   `frozen` depending on whether the community has proven via a challenge
//!   it's actively contributing to the network with infrastructure provisioning
//!   (i.e. a [collator][3] node) or by depositing funds.
//! - **Validity Challenge:** A proof that a community is actively contributing
//!   to the network. It's possible for a trusted origin to manually mark a
//!   community challenge as passed, effectively changing the status of the
//!   community to `active`.
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
//! - **Voting Method:** Can be either rank weighed, member-counted, or
//!   asset-weighed and determines how the votes of proposals will be tallied.
//!
//! ## Lifecycle
//!
//! ```ignore
//! [       ] --> [Awaiting]              --> [Active]            --> [Frozen]      --> [Blocked]
//! apply_for     set_metadata                set_metadata            set_metadata      unblock
//!               fulfill_challenge           block                   block
//!               force_complete_challenge    add_member              thaw
//!                                           remove_member
//!                                           promote_member
//!                                           demote_member
//!                                           open_proposal
//!                                           vote_proposal
//!                                           close_proposal
//!                                           set_admin
//!                                           set_voting_mechanism
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
//! - [`apply_for`][c00]: Registers an appliation as a new community, taking an
//!   [existential deposit][8] used to create the community account.
//!
//! ### Permissioned Functions
//!
//! Calling these functions requires being a member of the community.
//!
//! - `fulfill_challenge`: Submit the challenge proof to validate the
//!   contribution status of the community.
//! - [`add_member`][c02]: Enroll an account as a community member. In theory,
//!   any community member should be able to add a member. However, this can be
//!   changed to ensure it is a privileged function.
//! - `open_proposal`: Creates a proposal to be voted by the community. At
//! this point, there can only be a single proposal at a time.
//! - `vote_proposal`: Adds a vote into a community proposal.
//!
//! ### Privileged Functions
//!
//! These functions can be called either by the community _admin_ or
//! dispatched through an approved proposal. !
//! - [`set_metadata`][c01]: Sets some [`CommunityMetadata`][t01] to describe
//!   the
//! community.
//! - [`remove_member`][c03]: Removes an account as a community member. While
//!   enrolling a member into the community can be an action taken by any
//!   member, the decision to remove a member should not be taken arbitrarily by
//!   any community member. Also, it shouldn't be possible to arbitrarily remove
//!   the community admin, as some privileged calls would be impossible execute
//!   thereafter.
//! - `promote_member`: Increases the rank of a member in the community. ! -
//!   `demote_member`: Decreases the rank of a member in the community.
//! - `issue_token`: Creates a token that is either governance (only one per
//!   community allowed) or economic. While the first economic token is
//!   _"free"_," further ones would be subject to network-wide referenda.
//! - `close_proposal`: Forcefully closes a proposal, dispatching the call when
//!   approved.
//! - `set_sufficient_asset`: Marks an [asset][7] issued by the community as
//!   sufficient. Only one asset at a time can be marked as such.
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
//! - [`community`][g00]: Stores the basic information of the community. If a
//!   value exists for a specified [`ComumunityId`][t00], this means a community
//!   exists.
//! - [`metadata`][g01]: Stores the metadata regarding a community.
//! - [`membership`][g02]: Stores the information of a community (specified by
//!   its [`CommunityId`][t00]) member (specified by it's [`AccountId`][1]).
//! - [`members_count`][g03]: Stores the count of community members. This
//!   simplifies the process of keeping track of members' count.
//!
//! <!-- References -->
//! [1]: `frame_system::Config::AccountId`
//! [2]: https://h3geo.org/docs/highlights/indexing
//! [3]: https://docs.substrate.io/reference/glossary/#collator
//! [4]: https://docs.substrate.io/reference/glossary/#call
//! [5]: https://github.com/virto-network/virto-node/tree/master/pallets/payments
//! [6]: https://github.com/virto-network/virto-node/pull/282
//! [7]: https://paritytech.github.io/substrate/master/pallet_assets/index.html#terminology
//! [8]: https://docs.substrate.io/reference/glossary/#existential-deposit
//!
//! [t00]: `Config::CommunityId`
//! [t01]: `types::CommunityMetadata`
//!
//! [c00]: `crate::Pallet::apply`
//! [c01]: `crate::Pallet::set_metadata`
//! [c02]: `crate::Pallet::add_member`
//! [c03]: `crate::Pallet::remove_member`
//!
//! [g00]: `crate::Pallet::community`
//! [g01]: `crate::Pallet::metadata`
//! [g02]: `crate::Pallet::membership`
//! [g03]: `crate::Pallet::members_count`
pub use pallet::*;
pub use types::RawOrigin;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod functions;

pub mod traits;
pub mod types;
pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::{DispatchResult, *},
		traits::{
			fungible::{Inspect, Mutate},
			fungibles,
			tokens::Preservation,
			Polling,
		},
		Blake2_128Concat, Parameter,
	};
	use frame_system::{
		ensure_signed,
		pallet_prelude::{OriginFor, *},
	};
	use sp_runtime::traits::StaticLookup;
	use traits::member_rank;
	use types::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it
	/// depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// This type represents an unique ID for the community
		type CommunityId: Default + Parameter + MaxEncodedLen;

		/// This type represents a rank for a member in a community
		type Membership: Default + Parameter + MaxEncodedLen;

		type MemberRank: Default + Parameter + MaxEncodedLen + member_rank::Inspect + member_rank::Mutate;

		/// The asset used for governance
		type Assets: fungibles::Inspect<Self::AccountId>;

		/// Type represents interactions between fungibles (i.e. assets)
		type Balances: Inspect<Self::AccountId> + Mutate<Self::AccountId>;

		/// Because this pallet emits events, it depends on the runtime's
		/// definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;

		type Polls: Polling<Tally<Self>, Votes = VoteWeight, Moment = BlockNumberFor<Self>>;

		/// The pallet id used for deriving sovereign account IDs.
		#[pallet::constant]
		type PalletId: Get<frame_support::PalletId>;
	}

	/// The origin of the pallet
	#[pallet::origin]
	pub type Origin<T> = types::RawOrigin<CommunityIdOf<T>>;

	/// Stores the basic information of the community. If a value exists for a
	/// specified [`ComumunityId`][`Config::CommunityId`], this means a
	/// community exists.
	#[pallet::storage]
	#[pallet::getter(fn community)]
	pub(super) type Info<T> = StorageMap<_, Blake2_128Concat, CommunityIdOf<T>, CommunityInfo>;

	/// Stores the metadata regarding a community.
	#[pallet::storage]
	#[pallet::getter(fn metadata)]
	pub(super) type Metadata<T: Config> = StorageMap<_, Blake2_128Concat, CommunityIdOf<T>, CommunityMetadata>;

	/// Stores the membership information of a community (specified by its
	/// [`CommunityId`][`Config::CommunityId`]) member (specified by it's
	/// [`AccountId`][`frame_system::Config::AccountId`]).
	#[pallet::storage]
	#[pallet::getter(fn membership)]
	pub(super) type Members<T> =
		StorageDoubleMap<_, Blake2_128Concat, CommunityIdOf<T>, Blake2_128Concat, AccountIdOf<T>, MembershipOf<T>>;

	/// Stores the rank of a community (specified by its
	/// [`CommunityId`][`Config::CommunityId`]) member (specified by it's
	/// [`AccountId`][`frame_system::Config::AccountId`]).
	#[pallet::storage]
	#[pallet::getter(fn member_rank)]
	pub(super) type MemberRanks<T> =
		StorageDoubleMap<_, Blake2_128Concat, CommunityIdOf<T>, Blake2_128Concat, AccountIdOf<T>, MemberRankOf<T>>;

	/// Stores the count of community members. This simplifies the process of
	/// keeping track of members' count.
	#[pallet::storage]
	#[pallet::getter(fn members_count)]
	pub(super) type MembersCount<T> = StorageMap<_, Blake2_128Concat, CommunityIdOf<T>, u128>;

	/// Stores the governance strategy for the community.
	#[pallet::storage]
	#[pallet::getter(fn governance_strategy)]
	pub(super) type GovernanceStrategy<T> =
		StorageMap<_, Blake2_128Concat, CommunityIdOf<T>, CommunityGovernanceStrategy<AccountIdOf<T>, AssetIdOf<T>>>;

	/// Stores the list of votes for a community.
	#[pallet::storage]
	pub(super) type CommunityVotes<T> =
		StorageDoubleMap<_, Blake2_128Concat, CommunityIdOf<T>, Blake2_128Concat, AccountIdOf<T>, ()>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A [`Commmunity`][`types::Community`] has been created.
		CommunityCreated { id: T::CommunityId, who: T::AccountId },
		/// Some [`CommmuniMetadata`][`types::CommunityMetadata`] has been set
		/// for a community.
		MetadataSet {
			id: T::CommunityId,
			name: Option<ConstSizedField<64>>,
			description: Option<ConstSizedField<256>>,
			main_url: Option<ConstSizedField<256>>,
		},
		/// A proposal has been enqueued and is ready to be
		/// decided whenever it comes to the head of the comomunity
		/// proposals queue
		ProposalEnqueued {
			community_id: CommunityIdOf<T>,
			proposer: AccountIdOf<T>,
		},
	}

	// Errors inform users that something worked or went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The community doesn't exist in storage, nor they have members.
		CommunityDoesNotExist,
		/// A community with the same [`CommunityId`][`Config::CommunityId`]
		/// already exists, therefore cannot be applied for.
		CommunityAlreadyExists,
		/// The specified [`CommunityId`][`Config::CommunityId`] is not
		/// currently active
		CommunityNotActive,
		/// The specified [`AccountId`][`frame_system::Config::AccountId`] is
		/// not a member of the community
		NotAMember,
		/// The specified [`AccountId`][`frame_system::Config::AccountId`] is
		/// already a member of the community
		AlreadyAMember,
		/// It is not possible to remove the sole admin for a specified
		/// [`CommunityId`][`Config::CommunityId`], especially if it's the
		/// only member remaining. Please consider changing the admin first.
		CannotRemoveAdmin,
		/// The origin specified to propose the call execution is invalid.
		InvalidProposalOrigin,
		/// It is not possible to encode the call into a preimage.
		CannotEncodeCall,
		/// It is not possible to enqueue a proposal for a community
		CannotEnqueueProposal,
		/// The community has exceeded the max amount of enqueded proposals at
		/// this moment.
		ExceededMaxProposals,
		/// A member cannot be promoted since their rank is already at the upper
		/// bound. The only option available so far is creating a higher rank.
		ExceededPromoteBound,
		/// A member cannot be demoted since their rank is already at the upper
		/// bound. The options available so far are creating a lower rank, or
		/// removing the member.
		ExceededDemoteBound,
		/// It is not possible to dequeue a proposal
		CannotDequeueProposal,
		/// A call for the spciefied [Hash][`frame_system::Config::Hash`] is not
		/// found
		CannotFindCall,
		/// The poll the caller is trying to open is already opened.
		PollAlreadyOpened,
		/// The poll the caller is trying to close is already closed.
		PollAlreadyClosed,
		/// The criteria needed to close the poll is not met
		CannotClosePoll,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke
	// state changes. These functions materialize as "extrinsics", which are often
	// compared to transactions. Dispatchable functions must be annotated with a
	// weight and must return a DispatchResult.
	#[pallet::call(weight(<T as Config>::WeightInfo))]
	impl<T: Config> Pallet<T> {
		/// Registers an appliation as a new community, taking an
		/// [existential deposit][8] used to create the community account.
		///
		/// [8]: https://docs.substrate.io/reference/glossary/#existential-deposit
		#[pallet::call_index(0)]
		pub fn apply_for(origin: OriginFor<T>, community_id: T::CommunityId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_register_community(&who, &community_id)?;
			let community_account_id = Self::get_community_account_id(&community_id);
			let minimum_balance = T::Balances::minimum_balance();
			T::Balances::transfer(&who, &community_account_id, minimum_balance, Preservation::Preserve)?;

			Self::deposit_event(Event::CommunityCreated { id: community_id, who });
			Ok(())
		}

		/// Sets some [`CommunityMetadata`][11] to describe the
		/// community.
		///
		/// [11]: `types::CommunityMetadata`
		#[pallet::call_index(1)]
		pub fn set_metadata(
			origin: OriginFor<T>,
			community_id: T::CommunityId,
			name: Option<ConstSizedField<64>>,
			description: Option<ConstSizedField<256>>,
			url: Option<ConstSizedField<256>>,
		) -> DispatchResult {
			// Ensures caller is a privileged origin
			Self::ensure_origin_privileged(origin, &community_id)?;

			let metadata = Self::metadata(&community_id).unwrap_or_default();

			// Deposits metadata
			Self::do_set_metadata(
				&community_id,
				CommunityMetadata {
					name: name.clone().unwrap_or(metadata.name),
					description: description.clone().unwrap_or(metadata.description),
					main_url: url.clone().unwrap_or(metadata.main_url),
				},
			)?;

			// Emit an event.
			Self::deposit_event(Event::MetadataSet {
				id: community_id,
				name,
				description,
				main_url: url,
			});

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// === Memberships management ===

		/// Enroll an account as a community member. In theory,
		/// any community member should be able to add a member. However, this
		/// can be changed to ensure it is a privileged function.
		#[pallet::call_index(2)]
		pub fn add_member(
			origin: OriginFor<T>,
			community_id: T::CommunityId,
			who: AccountIdLookupOf<T>,
		) -> DispatchResult {
			Self::ensure_origin_member(origin, &community_id)?;
			Self::ensure_active(&community_id)?;

			let who = <<T as frame_system::Config>::Lookup as StaticLookup>::lookup(who)?;
			Self::do_insert_member(&community_id, &who)?;

			Ok(())
		}

		/// Removes an account as a community member. While
		/// enrolling a member into the community can be an action taken by any
		/// member, the decision to remove a member should not be taken
		/// arbitrarily by any community member. Also, it shouldn't be possible
		/// to arbitrarily remove the community admin, as some privileged calls
		/// would be impossible to execute thereafter.
		#[pallet::call_index(3)]
		pub fn remove_member(
			origin: OriginFor<T>,
			community_id: T::CommunityId,
			who: AccountIdLookupOf<T>,
		) -> DispatchResult {
			Self::ensure_origin_privileged(origin, &community_id)?;
			Self::ensure_active(&community_id)?;

			let who = <<T as frame_system::Config>::Lookup as StaticLookup>::lookup(who)?;
			Self::do_remove_member(&community_id, &who)?;

			Ok(())
		}

		// === Governance ===

		// ///
		// #[pallet::call_index(4)]
		// pub fn vote(
		// 	origin: OriginFor<T>,
		// 	#[pallet::compact] poll_index: PollIndexOf<T>,
		// 	_vote: VoteOf<T>,
		// ) -> DispatchResult {
		// 	let _ = ensure_signed(origin)?;
		// 	// TODO
		// 	Ok(())
		// }
	}
}
