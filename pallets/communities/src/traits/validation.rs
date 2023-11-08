use frame_support::{
	pallet_prelude::{Decode, Encode},
	Parameter,
};
use parity_scale_codec::MaxEncodedLen;
use sp_runtime::traits::Saturating;

#[derive(Debug, Encode, Decode)]
pub enum ChallengeRegistrationRejectionCause {
	/// The entity does not exist
	EntityDoesNotExist,
	/// Cannot register a challenge for the entity, since the maximum amount of
	/// registered challenges has been exceeded.
	TooManyRegisteredChallenges,
	/// Another challenge claimed by the registrar for the entity is already
	/// active.
	ChallengeAlreadyActiveForEntity,
	/// The entity has a current blockage, therefore it's not possible to alter
	/// its state.
	EntityBlocked,
}

#[derive(Debug, Encode, Decode)]
pub enum ValidationRejectionCause {
	/// The entity does not exist
	EntityDoesNotExist,
	/// A challenge for the entity claimed by the specified registrar does not
	/// exist
	ChallengeForRegistrarNotFound,
	/// The validation is not possible due to internal reasons on the entity end
	Other,
}

/// Handles registering challenges on behalf of an entity
pub trait ValidationChallenge {
	type EntityId: Parameter + MaxEncodedLen;
	type ChallengeRegistrarId: Parameter + MaxEncodedLen;
	type BlockNumber: Saturating;

	/// Registers a challenge for an entity to be fulfilled
	fn register_challenge(
		registrar_id: Self::ChallengeRegistrarId,
		entity_id: Self::EntityId,
	) -> Result<(), ChallengeRegistrationRejectionCause>;

	/// Indicates whether the given entity passed the challenge, giving a reason
	/// why dit did or did not.
	fn validate_challenge(
		registrar_id: Self::ChallengeRegistrarId,
		entity_id: Self::EntityId,
		passed: bool,
		reason: Option<Vec<u8>>,
	) -> Result<(), ValidationRejectionCause>;
}
