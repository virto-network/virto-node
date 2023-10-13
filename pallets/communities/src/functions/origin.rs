use super::*;
use frame_support::traits::{EnsureOrigin, OriginTrait};

impl<T: Config> Pallet<T> {
	pub(crate) fn get_origin(community_id: &CommunityIdOf<T>) -> Result<RawOrigin<T>, DispatchError> {
		let governance_strategy =
			GovernanceStrategy::<T>::get(community_id).ok_or(Error::<T>::CommunityDoesNotExist)?;

		Ok(RawOrigin::<T> {
			community_id: community_id.clone(),
			body_part: match governance_strategy {
				CommunityGovernanceStrategy::AdminBased(_) => BodyPart::Voice,
				CommunityGovernanceStrategy::MemberCountPoll { min } => BodyPart::Members { min },
				CommunityGovernanceStrategy::AssetWeighedPoll {
					asset_id: _,
					num,
					denum,
				} => BodyPart::Fraction { num, denum },
				CommunityGovernanceStrategy::RankedWeighedPoll { num, denum } => BodyPart::Fraction { num, denum },
			},
		})
	}
}
