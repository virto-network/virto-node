use super::*;

#[repr(u16)]
#[derive(TypeInfo, Encode, Decode, Clone, Debug, PartialEq, TryFromPrimitive)]
pub enum AssetsApiError {
	// The error codes are part of the ABI: only ever append new variants.
	CannotDeposit,
	CannotTransfer,
	/// The asset could not be minted.
	CannotMint,
	/// The asset could not be burnt.
	CannotBurn,
	/// The calling contract does not belong to a group (it was not
	/// instantiated through the contracts store).
	NotAGroupContract,
	/// The contract's group does not hold the role the operation requires on
	/// the asset.
	NotAuthorised,
	/// Contracts may not mint or burn this asset (on Kreivo: it is not a
	/// local asset).
	NotALocalAsset,
}

impl From<AssetsApiError> for KreivoApisError {
	fn from(error: AssetsApiError) -> Self {
		KreivoApisError::Assets(error)
	}
}
