use crate::Config;
use frame_support::pallet_prelude::*;
use frame_support::traits::{
	fungible, fungibles,
	membership::{GenericRank, Membership, WithRank},
	Polling, VoteTally,
};
use sp_runtime::traits::StaticLookup;

pub type AssetIdOf<T> = <<T as Config>::Assets as fungibles::Inspect<AccountIdOf<T>>>::AssetId;
pub type AssetBalanceOf<T> = <<T as Config>::Assets as fungibles::Inspect<AccountIdOf<T>>>::Balance;
pub type NativeBalanceOf<T> = <<T as Config>::Balances as fungible::Inspect<AccountIdOf<T>>>::Balance;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type CommunityIdOf<T> = <T as Config>::CommunityId;
pub type VoteOf<T> = Vote<AssetIdOf<T>, AssetBalanceOf<T>>;
pub type PollIndexOf<T> = <<T as Config>::Polls as Polling<Tally<T>>>::Index;
pub type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
pub type PalletsOriginOf<T> =
	<<T as frame_system::Config>::RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin;

pub type SizedField<S> = BoundedVec<u8, S>;
pub type ConstSizedField<const S: u32> = SizedField<ConstU32<S>>;

/// The Community struct holds the basic definition of a community. It includes
/// the current state of a community, the [`AccountId`][1] for the community
/// admin, and (if any) the ID of the community-issued asset the community has
/// marked to be sufficient.
///
/// [1]: `frame_system::Config::AccountId`
#[derive(Decode, Default, Encode, MaxEncodedLen, TypeInfo)]
pub struct CommunityInfo {
	/// The current state of the community.
	pub state: CommunityState,
}

/// The current state of the community. It represents whether a community
/// is awaiting to prove their contribution to the network, is active
/// and can operate, blocked due to a violation of network norms, or
/// it's being frozen by the community administrators.
#[derive(Decode, Default, Encode, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum CommunityState {
	/// The community is opperating normally.
	#[default]
	Active,
	/// The community is blocked, typically as a result of a restriction imposed
	/// by violating the norms of the network.
	Blocked,
}

/// The CommunityMetadata struct stores some descriptive information about
/// the community.
#[derive(Clone, Debug, Decode, Default, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct CommunityMetadata {
	/// The name of the community
	pub name: ConstSizedField<64>,
	/// A short description of the community
	pub description: ConstSizedField<256>,
	/// The main URL that can lead to information about the community
	pub main_url: ConstSizedField<256>,
}

pub(crate) type MembershipIdPart = u32;

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct MembershipId<CommunityId>(pub(crate) CommunityId, pub(crate) MembershipIdPart);

impl<CommunityId> From<(CommunityId, MembershipIdPart)> for MembershipId<CommunityId> {
	fn from(value: (CommunityId, MembershipIdPart)) -> Self {
		MembershipId(value.0, value.1)
	}
}

#[derive(Decode, Encode, TypeInfo)]
pub struct MembershipInfo<CommunityId> {
	id: MembershipId<CommunityId>,
	rank: GenericRank,
}

impl<CommunityId> MembershipInfo<CommunityId> {
	pub fn community(&self) -> &CommunityId {
		&self.id.0
	}
}

impl<CommunityId> Membership for MembershipInfo<CommunityId>
where
	CommunityId: Parameter + 'static,
{
	type Id = MembershipId<CommunityId>;

	fn new(id: Self::Id) -> Self {
		Self {
			id,
			rank: Default::default(),
		}
	}

	fn id(&self) -> Self::Id {
		self.id
	}
}

impl<CommunityId> WithRank for MembershipInfo<CommunityId>
where
	CommunityId: Parameter + 'static,
{
	fn rank(&self) -> GenericRank {
		self.rank
	}

	fn set_rank(&mut self, rank: impl Into<GenericRank>) {
		self.rank = rank.into();
	}
}

// Governance

pub type VoteWeight = u32;

///
#[derive(Clone, Debug, Decode, Encode, PartialEq, TypeInfo)]
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
#[derive(Clone, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
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
