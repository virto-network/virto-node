use crate::{
	types::{CommunityState::Active, MembershipIdPart},
	CommunityIdFor, Config, Info,
};
use core::marker::PhantomData;
use frame_support::{
	pallet_prelude::*,
	traits::{GenericRank, OriginTrait},
};
use sp_runtime::Permill;

pub struct EnsureCommunity<T>(PhantomData<T>);

impl<T> EnsureOrigin<T::RuntimeOrigin> for EnsureCommunity<T>
where
	T::RuntimeOrigin:
		OriginTrait + Into<Result<RawOrigin<T::CommunityId>, T::RuntimeOrigin>> + From<RawOrigin<T::CommunityId>>,
	T: Config,
{
	type Success = T::CommunityId;

	fn try_origin(o: T::RuntimeOrigin) -> Result<Self::Success, T::RuntimeOrigin> {
		use frame_system::RawOrigin::{None, Root};
		if matches!(o.as_system_ref(), Some(Root) | Some(None)) {
			return Err(o);
		}
		let id = match o.clone().into() {
			Ok(RawOrigin { community_id, .. }) => community_id,
			Err(_) => {
				let origin = o.clone().into_caller();
				CommunityIdFor::<T>::get(origin).ok_or_else(|| o.clone())?
			}
		};
		Info::<T>::get(id)
			.and_then(|c| c.state.eq(&Active).then_some(id))
			.ok_or(o)
	}
}

/// Origin to represent the voice of a community or a subset of its members
/// as well as the voting preference of said group.
#[derive(TypeInfo, Encode, Decode, MaxEncodedLen, Clone, Eq, PartialEq, Debug)]
pub struct RawOrigin<CommunityId> {
	community_id: CommunityId,
	method: DecisionMethod,
	subset: Option<Subset>,
}

impl<CommunityId: Clone> RawOrigin<CommunityId> {
	pub const fn new(community_id: CommunityId) -> Self {
		RawOrigin {
			community_id,
			method: DecisionMethod::Membership,
			subset: None,
		}
	}

	pub fn with_subset(&mut self, s: Subset) {
		self.subset = Some(s);
	}

	pub fn id(&self) -> CommunityId {
		self.community_id.clone()
	}
	pub fn decision_method(&self) -> DecisionMethod {
		self.method.clone()
	}
}

/// Subsets of the community can also have a voice
#[derive(Clone, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum Subset {
	Member(MembershipIdPart),
	Members { count: u32 },
	Fraction(Permill),
	AtLeastRank(GenericRank),
}

/// The mechanism used by the community or one of its subsets to make decisions
#[derive(Clone, Debug, Decode, Default, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum DecisionMethod {
	#[default]
	Membership,
	NativeToken,
	CommunityAsset,
	Rank,
}

#[cfg(feature = "xcm")]
impl<Id: Into<u32>> TryFrom<RawOrigin<Id>> for xcm::v3::Junction {
	type Error = ();

	fn try_from(o: RawOrigin<Id>) -> Result<Self, Self::Error> {
		use xcm::v3::{BodyId, BodyPart, Junction::Plurality};
		let part = match o.subset {
			None => BodyPart::Voice,
			Some(Subset::Member(_)) => BodyPart::Members { count: 1 },
			Some(Subset::Members { count }) => BodyPart::Members { count },
			Some(Subset::Fraction(per)) => BodyPart::Fraction {
				nom: per.deconstruct(),
				denom: <Permill as sp_runtime::PerThing>::ACCURACY,
			},
			Some(Subset::AtLeastRank(_)) => return Err(()),
		};
		Ok(Plurality {
			id: BodyId::Index(o.community_id.into()),
			part,
		})
	}
}

#[cfg(feature = "xcm")]
impl<Id: From<u32> + Clone> TryFrom<xcm::v3::Junction> for RawOrigin<Id> {
	type Error = ();

	fn try_from(value: xcm::v3::Junction) -> Result<Self, Self::Error> {
		use xcm::v3::{BodyId::Index, BodyPart::*, Junction::Plurality};
		let Plurality { id: Index(id), part } = value else {
			return Err(());
		};
		let subset = match part {
			Voice => None,
			Members { count } => Some(Subset::Members { count }),
			Fraction { nom, denom } => Some(Subset::Fraction(Permill::from_rational(nom, denom))),
			_ => return Err(()),
		};
		let mut origin = RawOrigin::new(id.into());
		if let Some(s) = subset {
			origin.with_subset(s);
		}
		Ok(origin)
	}
}
