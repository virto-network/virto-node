use super::*;

use core::marker::PhantomData;
use fc_traits_memberships::{Inspect, Manager, Rank};
use frame_support::traits::{
	nonfungible_v2::Inspect as NonFunInspect,
	nonfungibles_v2::{Inspect as NonFunsInspect, Mutate},
};
use sp_runtime::DispatchError;

pub struct KreivoMemberships<F, M>(PhantomData<(F, M)>);

impl<F, M, AccountId> Inspect<AccountId> for KreivoMemberships<F, M>
where
	F: Inspect<AccountId>,
{
	type Group = F::Group;
	type Membership = F::Membership;

	fn user_memberships(
		who: &AccountId,
		maybe_group: Option<Self::Group>,
	) -> Box<dyn Iterator<Item = (Self::Group, Self::Membership)>> {
		F::user_memberships(who, maybe_group)
	}

	fn check_membership(who: &AccountId, m: &Self::Membership) -> Option<Self::Group> {
		F::check_membership(who, m)
	}

	fn members_total(group: &Self::Group) -> u32 {
		F::members_total(group)
	}
}

const WELL_KNOWN_ATTR_KEYS: [&[u8]; 3] = [
	&*b"membership_member_rank",
	&*b"membership_gas",
	&*b"membership_expiration",
];

impl<F, M, AccountId> Manager<AccountId, pallet_nfts::ItemConfig> for KreivoMemberships<F, M>
where
	F: Inspect<AccountId>
		+ Manager<AccountId, pallet_nfts::ItemConfig>
		+ NonFunsInspect<AccountId>
		+ Mutate<AccountId, pallet_nfts::ItemConfig, CollectionId = Self::Group, ItemId = Self::Membership>,
	M: NonFunInspect<AccountId, ItemId = Self::Membership>,
{
	fn assign(group: &Self::Group, m: &Self::Membership, who: &AccountId) -> Result<(), DispatchError> {
		F::assign(group, m, who)?;
		for key in WELL_KNOWN_ATTR_KEYS.into_iter() {
			if let Some(value) = M::system_attribute(m, key) {
				F::set_attribute(group, m, key, &value)?;
			}
		}
		Ok(())
	}

	fn release(group: &Self::Group, m: &Self::Membership) -> Result<(), DispatchError> {
		F::release(group, m)
	}
}

impl<F, M, AccountId, R> Rank<AccountId, pallet_nfts::ItemConfig, R> for KreivoMemberships<F, M>
where
	F: Inspect<AccountId> + Rank<AccountId, pallet_nfts::ItemConfig, R>,
	R: Eq + Ord,
{
	fn rank_of(group: &Self::Group, m: &Self::Membership) -> Option<R> {
		F::rank_of(group, m)
	}

	fn set_rank(group: &Self::Group, m: &Self::Membership, rank: impl Into<R>) -> Result<(), DispatchError> {
		F::set_rank(group, m, rank)
	}

	fn ranks_total(group: &Self::Group) -> u32 {
		F::ranks_total(group)
	}
}
