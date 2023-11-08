use crate::traits::validation::{ChallengeRegistrationRejectionCause, ValidationChallenge, ValidationRejectionCause};
use core::marker::PhantomData;
use frame_support::pallet_prelude::*;
use sp_runtime::ModuleError;

pub(crate) trait Config: frame_system::Config {
	type EntityId: Parameter + MaxEncodedLen;
	type Challenge: ValidationChallenge;
	type RegistrarId: Get<<Self::Challenge as ValidationChallenge>::ChallengeRegistrarId>;
}

pub struct Challenger<T: Config>(PhantomData<T>);

#[derive(Debug)]
pub(crate) enum Error<T: Config> {
	FailedRegistering(ChallengeRegistrationRejectionCause),
	FailedValidation(ValidationRejectionCause),
	_PhantomData(T),
}

impl<T: Config> From<Error<T>> for DispatchError {
	fn from(error: Error<T>) -> DispatchError {
		DispatchError::Module(ModuleError {
			index: 255,
			error: match error {
				Error::FailedRegistering(_) => 0u32.to_le_bytes(),
				Error::FailedValidation(_) => 1u32.to_le_bytes(),
				_ => u32::MAX.to_le_bytes(),
			},
			message: Some(&match error {
				Error::FailedRegistering(_) => "FailedRegistering",
				Error::FailedValidation(_) => "FailedValidation",
				_ => unimplemented!(),
			}),
		})
	}
}

impl<T: Config> Challenger<T> {
	/// Registers a challenge
	pub fn register(entity_id: <T::Challenge as ValidationChallenge>::EntityId) -> DispatchResult {
		T::Challenge::register_challenge(T::RegistrarId::get(), entity_id)
			.map_err(|e| Error::<T>::FailedRegistering(e).into())
	}

	/// Validates a challenge
	pub fn validate(entity_id: <T::Challenge as ValidationChallenge>::EntityId, passed: bool) -> DispatchResult {
		T::Challenge::validate_challenge(T::RegistrarId::get(), entity_id, passed, None)
			.map_err(|e| Error::<T>::FailedValidation(e).into())
	}
}
