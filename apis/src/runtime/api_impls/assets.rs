use super::*;

use config::{AssetBalanceOf, AssetIdOf, Config, GroupInfo};

use frame_support::sp_runtime::traits::Zero;
use frame_support::traits::fungibles::{roles, Inspect, Mutate};
use frame_support::traits::tokens::{Fortitude, Precision, Preservation, WithdrawConsequence};
use frame_support::traits::Contains;

/// A helper structure that implements [`AssetsAPI`] in the context of the
/// Runtime.
pub struct RuntimeAssetsAPI<T>(PhantomData<T>);

/// A role on an asset, as `pallet_assets` defines them.
#[derive(Clone, Copy)]
enum AssetRole {
	/// The account that may mint the asset.
	Issuer,
	/// The account that may burn the asset from any account.
	Admin,
}

impl<T: Config> RuntimeAssetsAPI<T> {
	/// Authorises the contract at `address` to act with a `role` on an `asset`.
	///
	/// It succeeds if the asset is one contracts may mint and burn
	/// ([`Config::MintableAssets`]), the contract belongs to a group, and that
	/// group's account holds the `role` on the `asset`. It looks at the
	/// contract's own address only: a contract called by another one
	/// authorises as itself, never as its caller.
	fn ensure_group_has_role(
		address: &AccountIdOf<T>,
		asset: &AssetIdOf<T>,
		role: AssetRole,
	) -> Result<(), KreivoApisError> {
		if !T::MintableAssets::contains(asset) {
			return Err(AssetsApiError::NotALocalAsset.into());
		}

		let group = T::GroupInfo::maybe_group(address).ok_or(AssetsApiError::NotAGroupContract)?;
		let group_account = T::GroupInfo::group_account(&group);

		let holder = match role {
			AssetRole::Issuer => <T::Assets as roles::Inspect<_>>::issuer(asset.clone()),
			AssetRole::Admin => <T::Assets as roles::Inspect<_>>::admin(asset.clone()),
		};

		if holder.as_ref() == Some(&group_account) {
			Ok(())
		} else {
			Err(AssetsApiError::NotAuthorised.into())
		}
	}

	/// Refuses a burn that `pallet_assets::burn` would refuse, but
	/// `fungibles::Mutate::burn_from` turns into burning nothing.
	///
	/// `pallet_assets::burn` fails if the asset is frozen or being destroyed,
	/// if `who` holds no account of the asset, or if `who`'s account is frozen
	/// or blocked. Through the trait, the reducible balance is zero in all
	/// those cases, so `burn_from` returns `Ok(0)` instead. A zero `amount` is
	/// a no-op the extrinsic lets through on a frozen asset as well.
	fn ensure_can_burn(
		asset: &AssetIdOf<T>,
		who: &AccountIdOf<T>,
		amount: AssetBalanceOf<T>,
	) -> Result<(), KreivoApisError> {
		// Withdrawing nothing only checks the asset's status: `Frozen` if it is
		// frozen, `UnknownAsset` if it is being destroyed.
		match T::Assets::can_withdraw(asset.clone(), who, Zero::zero()) {
			WithdrawConsequence::Success => {}
			WithdrawConsequence::Frozen if amount.is_zero() => {}
			_ => return Err(AssetsApiError::CannotBurn.into()),
		}

		if amount.is_zero() {
			return Ok(());
		}

		let reducible = T::Assets::reducible_balance(asset.clone(), who, Preservation::Expendable, Fortitude::Polite);
		if reducible.is_zero() {
			let balance = T::Assets::balance(asset.clone(), who);
			// No balance: `who` has no account to burn from. A frozen or blocked
			// account cannot withdraw any of its balance.
			if balance.is_zero()
				|| matches!(
					T::Assets::can_withdraw(asset.clone(), who, balance),
					WithdrawConsequence::Frozen
				) {
				return Err(AssetsApiError::CannotBurn.into());
			}
		}

		Ok(())
	}
}

impl<T, E> AssetsAPI<E> for RuntimeAssetsAPI<T>
where
	T: Config,
	E: Ext<T = T>,
{
	type AccountId = T::AccountId;
	type AssetId = <T::Assets as Inspect<T::AccountId>>::AssetId;
	type Balance = <T::Assets as Inspect<T::AccountId>>::Balance;

	fn balance(_: &E, asset: Self::AssetId, who: &Self::AccountId) -> Self::Balance {
		T::Assets::balance(asset, who)
	}

	fn deposit(e: &E, asset: Self::AssetId, amount: Self::Balance) -> Result<Self::Balance, KreivoApisError> {
		let caller = e
			.caller()
			.account_id()
			.map_err(|_| KreivoApisError::ExtQueryError)?
			.clone();
		T::Assets::transfer(asset, &caller, e.address(), amount, Preservation::Preserve)
			.map_err(|_| AssetsApiError::CannotDeposit.into())
	}

	fn transfer(
		e: &E,
		asset: Self::AssetId,
		amount: Self::Balance,
		beneficiary: &Self::AccountId,
	) -> Result<Self::Balance, KreivoApisError> {
		T::Assets::transfer(asset, e.address(), beneficiary, amount, Preservation::Preserve)
			.map_err(|_| AssetsApiError::CannotTransfer.into())
	}

	/// Mirrors `pallet_assets::mint` called by the asset's issuer.
	///
	/// `fungibles::Mutate::mint_into` applies the same checks as the extrinsic
	/// (the asset must be live, a blocked account cannot receive, and a new
	/// account must reach the minimum balance), except for the issuer check,
	/// which [`RuntimeAssetsAPI::ensure_group_has_role`] makes instead.
	fn mint(
		e: &E,
		asset: Self::AssetId,
		amount: Self::Balance,
		beneficiary: &Self::AccountId,
	) -> Result<Self::Balance, KreivoApisError> {
		Self::ensure_group_has_role(e.address(), &asset, AssetRole::Issuer)?;
		T::Assets::mint_into(asset, beneficiary, amount).map_err(|_| AssetsApiError::CannotMint.into())
	}

	/// Mirrors `pallet_assets::burn` called by the asset's admin: best effort,
	/// and `who` may be reaped.
	///
	/// It uses `Fortitude::Polite`: the extrinsic never burns frozen funds (its
	/// reducible balance keeps whatever the asset's freezer holds), and
	/// `Polite` is the `Fortitude` that respects freezes. `pallet_assets`
	/// ignores the `Fortitude` argument anyway, so this only states the intent.
	fn burn(
		e: &E,
		asset: Self::AssetId,
		amount: Self::Balance,
		who: &Self::AccountId,
	) -> Result<Self::Balance, KreivoApisError> {
		Self::ensure_group_has_role(e.address(), &asset, AssetRole::Admin)?;
		Self::ensure_can_burn(&asset, who, amount)?;
		T::Assets::burn_from(
			asset,
			who,
			amount,
			Preservation::Expendable,
			Precision::BestEffort,
			Fortitude::Polite,
		)
		.map_err(|_| AssetsApiError::CannotBurn.into())
	}
}
