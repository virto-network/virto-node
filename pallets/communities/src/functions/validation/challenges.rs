use super::*;

use crate::traits::validation::{ChallengeRegistrationRejectionCause, ValidationChallenge, ValidationRejectionCause};

impl<T: Config> ValidationChallenge for Pallet<T> {
	type EntityId = T::CommunityId;
	type ChallengeRegistrarId = T::ChallengeRegistrarId;
	type BlockNumber = BlockNumberFor<T>;

	/// Registers a challenge claim for a community. This will change the state
	/// of such commnunity depending on the current state, or would reject the
	/// change of state if the community is not active.
	fn register_challenge(
		registrar_id: Self::ChallengeRegistrarId,
		community_id: Self::EntityId,
	) -> Result<(), ChallengeRegistrationRejectionCause> {
		let info = Self::community(community_id).ok_or(ChallengeRegistrationRejectionCause::EntityDoesNotExist)?;

		if info.state == CommunityState::Suspended {
			Err(ChallengeRegistrationRejectionCause::EntityBlocked)?;
		}

		if Challenges::<T>::contains_key(community_id, &registrar_id) {
			Err(ChallengeRegistrationRejectionCause::ChallengeAlreadyActiveForEntity)?;
		}

		Challenges::<T>::insert(community_id, &registrar_id, ());

		Ok(())
	}

	fn validate_challenge(
		registrar_id: Self::ChallengeRegistrarId,
		community_id: Self::EntityId,
		passed: bool,
		_reason: Option<Vec<u8>>,
	) -> Result<(), ValidationRejectionCause> {
		Info::<T>::try_mutate(community_id, |info| {
			let info = info.as_mut().ok_or(ValidationRejectionCause::EntityDoesNotExist)?;

			Challenges::<T>::try_mutate_exists(community_id, registrar_id, |challenge| {
				if challenge.as_mut().is_none() {
					Err(ValidationRejectionCause::ChallengeForRegistrarNotFound)?;
				}

				info.state = if passed {
					CommunityState::Active
				} else {
					CommunityState::FailedChallenge
				};

				*challenge = None;
				Ok(())
			})
		})
	}
}
