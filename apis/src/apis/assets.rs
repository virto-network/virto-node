//! # Assets APIs
//!
//! Facilitate transactions using arbitrary assets between external callers and
//! the application.
//!
//! ## Methods
//!
//! The supported methods are:
//!
//! - **[`deposit`][AssetsAPI::deposit]:** Receives an `amount` of a certain
//!   `asset` from the caller of the application. Then, deposits that amount
//!   into the balance of the application asset account.
//! - **[`transfer`][AssetsAPI::transfer]:** Transfers an `amount` of a certain
//!   `asset` to a `beneficiary`.
//! - **[`mint`][AssetsAPI::mint]:** Mints an `amount` of an `asset` the
//!   application's group issues into a `beneficiary`.
//! - **[`burn`][AssetsAPI::burn]:** Burns up to an `amount` of an `asset` the
//!   application's group administers from `who`.

use crate::apis::error::KreivoApisError;
use core::fmt;
use frame_support::Parameter;
use parity_scale_codec::{Codec, EncodeLike};

/// An API for transacting with arbitrary assets.
pub trait AssetsAPI<Ext> {
	type AccountId: Codec + EncodeLike + Clone + Eq + fmt::Debug;
	type AssetId: Parameter;
	type Balance: Parameter + Copy;

	/// Returns the balance of an asset account.
	fn balance(e: &Ext, asset: Self::AssetId, who: &Self::AccountId) -> Self::Balance;

	/// Receives an `amount` of a certain `asset` from the caller of the
	/// application. Then, deposits that amount into the balance of the
	/// application asset account.
	fn deposit(e: &Ext, asset: Self::AssetId, amount: Self::Balance) -> Result<Self::Balance, KreivoApisError>;

	/// Transfers an `amount` of a certain `asset` to a `beneficiary`.
	fn transfer(
		e: &Ext,
		asset: Self::AssetId,
		amount: Self::Balance,
		beneficiary: &Self::AccountId,
	) -> Result<Self::Balance, KreivoApisError>;

	/// Mints an `amount` of a certain `asset` into a `beneficiary`, and returns
	/// the amount minted.
	///
	/// The `asset` must be one applications may mint (on Kreivo, a local
	/// asset), and the application must belong to a group whose account is the
	/// **issuer** of the `asset`. The application authorises as itself, regardless of who
	/// called it. Minting into an account that does not hold the `asset` yet
	/// must reach the asset's minimum balance.
	fn mint(
		e: &Ext,
		asset: Self::AssetId,
		amount: Self::Balance,
		beneficiary: &Self::AccountId,
	) -> Result<Self::Balance, KreivoApisError>;

	/// Burns up to an `amount` of a certain `asset` from `who`, and returns the
	/// amount burnt.
	///
	/// The `asset` must be one applications may burn (on Kreivo, a local
	/// asset), and the application must belong to a group whose account is the
	/// **admin** of the `asset`. The application authorises as itself, regardless of who
	/// called it. `who` can be any account. The burn is best effort: it burns
	/// as much of `amount` as `who` can lose, and may reap `who`'s account
	/// (if its remaining balance falls below the minimum, that rest is burnt
	/// too).
	fn burn(
		e: &Ext,
		asset: Self::AssetId,
		amount: Self::Balance,
		who: &Self::AccountId,
	) -> Result<Self::Balance, KreivoApisError>;
}
