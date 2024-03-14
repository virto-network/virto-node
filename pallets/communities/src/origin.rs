use crate::{
	types::{AssetIdOf, CommunityIdOf, CommunityState::Active, MembershipIdOf},
	CommunityIdFor, Config, Info,
};
use core::marker::PhantomData;
use frame_support::{
	pallet_prelude::*,
	traits::{membership::GenericRank, OriginTrait},
	PalletId,
};
use sp_runtime::{traits::AccountIdConversion, Permill};

pub struct EnsureCommunity<T>(PhantomData<T>);

impl<T> EnsureOrigin<T::RuntimeOrigin> for EnsureCommunity<T>
where
	T::RuntimeOrigin: OriginTrait + Into<Result<RawOrigin<T>, T::RuntimeOrigin>> + From<RawOrigin<T>>,
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

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<T::RuntimeOrigin, ()> {
		use crate::BenchmarkHelper;
		Ok(RawOrigin::new(T::BenchmarkHelper::community_id()).into())
	}
}

pub struct EnsureCommunityAccountId<T>(PhantomData<T>);

impl<T> EnsureOrigin<T::RuntimeOrigin> for EnsureCommunityAccountId<T>
where
	T::RuntimeOrigin: OriginTrait + From<frame_system::RawOrigin<T::AccountId>> + From<RawOrigin<T>>,
	T: Config,
{
	type Success = T::CommunityId;

	fn try_origin(o: T::RuntimeOrigin) -> Result<Self::Success, T::RuntimeOrigin> {
		match o.clone().into() {
			Ok(frame_system::RawOrigin::Signed(account_id)) => {
				let (_, community_id) = PalletId::try_from_sub_account(&account_id).ok_or(o.clone())?;
				Ok(community_id)
			}
			_ => Err(o),
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<T::RuntimeOrigin, ()> {
		use crate::BenchmarkHelper;
		Ok(RawOrigin::new(T::BenchmarkHelper::community_id()).into())
	}
}

/// Origin to represent the voice of a community or a subset of its members
/// as well as the voting preference of said group.
#[derive(TypeInfo, Encode, Decode, MaxEncodedLen, Clone, Eq, PartialEq, Debug)]
pub struct RawOrigin<T: Config> {
	community_id: CommunityIdOf<T>,
	method: DecisionMethod<AssetIdOf<T>>,
	subset: Option<Subset<T>>,
}

impl<T: Config> RawOrigin<T> {
	pub const fn new(community_id: CommunityIdOf<T>) -> Self {
		RawOrigin {
			community_id,
			method: DecisionMethod::Membership,
			subset: None,
		}
	}

	pub fn with_subset(&mut self, s: Subset<T>) {
		self.subset = Some(s);
	}

	pub fn with_decision_method(&mut self, m: DecisionMethod<AssetIdOf<T>>) {
		self.method = m;
	}

	pub fn id(&self) -> CommunityIdOf<T> {
		self.community_id
	}

	pub fn decision_method(&self) -> DecisionMethod<AssetIdOf<T>> {
		self.method.clone()
	}
}

/// Subsets of the community can also have a voice
#[derive(Clone, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum Subset<T: Config> {
	Member(MembershipIdOf<T>),
	Members { count: u32 },
	Fraction(Permill),
	AtLeastRank(GenericRank),
}

/// The mechanism used by the community or one of its subsets to make decisions
#[derive(Clone, Debug, Decode, Default, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum DecisionMethod<AssetId> {
	#[default]
	Membership,
	NativeToken,
	CommunityAsset(AssetId),
	Rank,
}

#[cfg(feature = "xcm")]
impl<T> TryFrom<RawOrigin<T>> for xcm::v3::Junction
where
	T: Config,
	u32: From<CommunityIdOf<T>>,
{
	type Error = ();

	fn try_from(o: RawOrigin<T>) -> Result<Self, Self::Error> {
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
impl<T: Config> TryFrom<xcm::v3::Junction> for RawOrigin<T>
where
	T: Config,
	T::CommunityId: From<u32> + From<u64>,
{
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
