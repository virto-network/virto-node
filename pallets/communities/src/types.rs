use core::ops::Deref;

use frame_support::pallet_prelude::{Decode, Encode};
use frame_support::traits::fungible::Inspect;
use frame_support::traits::{fungibles, OriginTrait, Polling, VoteTally};
use frame_support::{sp_runtime::BoundedVec, traits::ConstU32};
use parity_scale_codec::MaxEncodedLen;
use scale_info::{prelude::vec::Vec, TypeInfo};
use sp_runtime::traits::StaticLookup;

use crate::Config;
pub(crate) use frame_system::Config as SystemConfig;

pub type AssetIdOf<T> = <<T as Config>::Assets as fungibles::Inspect<AccountIdOf<T>>>::AssetId;
pub type AssetBalanceOf<T> = <<T as Config>::Assets as fungibles::Inspect<AccountIdOf<T>>>::Balance;
pub type NativeBalanceOf<T> = <<T as Config>::Balances as Inspect<AccountIdOf<T>>>::Balance;
pub type AccountIdOf<T> = <T as SystemConfig>::AccountId;
pub type AccountIdLookupOf<T> = <<T as SystemConfig>::Lookup as StaticLookup>::Source;
pub type CommunityIdOf<T> = <T as Config>::CommunityId;
pub type MemberListOf<T> = Vec<AccountIdOf<T>>;
pub type MembershipOf<T> = <T as Config>::Membership;
pub type VoteOf<T> = Vote<AssetIdOf<T>, AssetBalanceOf<T>>;
pub type PollIndexOf<T> = <<T as Config>::Polls as Polling<Tally<T>>>::Index;
pub type RuntimeOriginOf<T> = <<T as SystemConfig>::RuntimeOrigin as OriginTrait>::PalletsOrigin;

pub type SizedField<S> = BoundedVec<u8, S>;
pub type ConstSizedField<const S: u32> = SizedField<ConstU32<S>>;

/// The Community struct holds the basic definition of a community. It includes
/// the current state of a community, the [`AccountId`][1] for the community
/// admin, and (if any) the ID of the community-issued asset the community has
/// marked to be sufficient.
///
/// [1]: `frame_system::Config::AccountId`
#[derive(TypeInfo, Encode, Decode, MaxEncodedLen)]
pub struct CommunityInfo {
	/// The current state of the community.
	pub state: CommunityState,
}

/// The current state of the community. It represents whether a community
/// is awaiting to prove their contribution to the network, is active
/// and can operate, blocked due to a violation of network norms, or
/// it's being frozen by the community administrators.
#[derive(Default, TypeInfo, PartialEq, Encode, Decode, MaxEncodedLen)]
pub enum CommunityState {
	/// The community is currently awaiting to prove they are contributing
	/// to the network.
	#[default]
	Awaiting,
	/// The community has proven the required contributions to the network
	/// and can operate.
	Active,
	/// The community is frozen, and is temporality unable to operate. This
	/// state can be changed by thawing the community via a privileged call.
	Frozen,
	/// The community is blocked, typically as a result of a restriction imposed
	/// by violating the norms of the network. In practice, this is an
	/// equivalent to being `frozen`, howerver the state cannot be changed by
	/// the community administrators, but by submitting a proposal to the
	/// network for it to be changed.
	Blocked,
}

/// The CommunityMetadata struct stores some descriptive information about
/// the community.
#[derive(TypeInfo, Eq, PartialEq, Default, Debug, Clone, Encode, Decode, MaxEncodedLen)]
pub struct CommunityMetadata {
	/// The name of the community
	pub name: ConstSizedField<64>,
	/// A short description of the community
	pub description: ConstSizedField<256>,
	/// The main URL that can lead to information about the community
	pub main_url: ConstSizedField<256>,
}

/// A general purpose rank in the range 0-100
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Rank(u8);

impl Rank {
	pub const MAX: Self = Rank(100);
	pub const MIN: Self = Rank(0);

	#[inline]
	pub fn promote(&mut self) {
		*self = self.0.saturating_add(1).min(Self::MAX.0).into()
	}

	#[inline]
	pub fn demote(&mut self) {
		*self = self.0.saturating_sub(1).max(Self::MIN.0).into()
	}
}

impl From<u8> for Rank {
	fn from(rank: u8) -> Self {
		Rank(rank)
	}
}
impl Deref for Rank {
	type Target = u8;
	fn deref(&self) -> &Self::Target {
		&(self.0)
	}
}

// Governance

pub type VoteWeight = u32;
/// This structure holds a governance strategy. This defines how to behave
/// when ensuring privileged calls and deciding executing
/// calls
#[derive(TypeInfo, Encode, Decode, MaxEncodedLen, Clone, Eq, PartialEq, Debug)]
#[scale_info(skip_type_params(AccountId, AssetId))]
pub enum CommunityGovernanceStrategy<AccountId, AssetId> {
	/// The community governance lies in the shoulders of the admin of it.
	///
	/// This is equivalent to `RawOrigin::Member` on collectives-pallet, or
	/// `BodyPart::Voice` on XCM.
	AdminBased(AccountId),
	/// The community governance relies on a member count-based (one member,
	/// one vote) poll.
	///
	/// This is equivalent to `RawOrigin::Members` on collectives-pallet, or
	/// `BodyPart::Members` on XCM.
	MemberCountPoll { min: VoteWeight },
	/// The community governance relies on an asset-weighed (one token,
	/// one vote) poll.
	///
	/// This is equivalent to `RawOrigin::Members` on collectives-pallet, or
	/// `BodyPart::Fraction` on XCM.
	AssetWeighedPoll {
		asset_id: AssetId,
		min_approval: VoteWeight,
	},
	/// The community governance relies on an ranked-weighed (one member vote,
	/// the number of votes corresponding to the rank of member) poll,
	///
	/// This is equivalent to `RawOrigin::Members` on collectives-pallet, or
	/// `BodyPart::Fraction` on XCM.
	RankedWeighedPoll { min_approval: VoteWeight },
}

///
#[derive(TypeInfo, Encode, Decode, Debug, PartialEq, Clone)]
pub enum Vote<AssetId, AssetBalance> {
	AssetBalance(bool, AssetId, AssetBalance),
	Standard(bool),
}

impl<A, B> From<Vote<A, B>> for VoteWeight {
	fn from(_value: Vote<A, B>) -> Self {
		todo!()
	}
}

///
#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound(T: Config))]
pub struct Tally<T>(core::marker::PhantomData<T>);

impl<T: Config> VoteTally<VoteWeight, T::CommunityId> for Tally<T> {
	fn new(_: T::CommunityId) -> Self {
		todo!()
	}

	fn ayes(&self, _cid: T::CommunityId) -> VoteWeight {
		todo!()
	}

	fn support(&self, _cid: T::CommunityId) -> sp_runtime::Perbill {
		todo!()
	}

	fn approval(&self, _cid: T::CommunityId) -> sp_runtime::Perbill {
		todo!()
	}
}
