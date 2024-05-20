#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use pallet_payments::{AssetIdOf, BalanceOf, FeeHandler, Fees};
pub use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_runtime::traits::{AccountIdConversion, Zero};
use sp_runtime::Percent;

use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::tokens::Balance, PalletId};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	use sp_runtime::traits::{Member, Get};

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_payments::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type CommunityId: Member + Parameter + MaxEncodedLen + TypeInfo;
		type Fee: Balance + Parameter + MaxEncodedLen + From<u64> + TypeInfo;
		type MandatoryFee: Get<u64>;
		type CommunityPalletId: Get<frame_support::PalletId>;
		/// Origin authorized to manage memeberships of an active community
		type SetFeesOrigin: EnsureOrigin<OriginFor<Self>>;
		type CommunitiesPalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn community_fees)]
	pub type CommunityFees<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::CommunityId, // CommunityId
		Fees,   // Fees to pay struct,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Setted ZeroFees for a community
		ZeroFees { community_id: T::CommunityId },
		/// New fees defined for community
		SettedNewFees { community_id: T::CommunityId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The selected fee is invalid
		InvalidFee,
		/// Tried to set a fee above maximum possible fee
		AboveMaximumValue,
		/// Unexpected math error
		MathError,
		/// Community does not exist
		NonExistingCommunity,
		/// Setting same value
		SettingSameValue,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::default())]
		pub fn set_fee(
			origin: OriginFor<T>,
			community_id: T::CommunityId,
			sender_fee: T::Fee,
			receiver_fee: T::Fee,
		) -> DispatchResultWithPostInfo {

			T::SetFeesOrigin::ensure_origin(origin)?;
			// Validate fees
			ensure!(
				sender_fee >= Zero::zero() && receiver_fee >= Zero::zero(),
				Error::<T>::InvalidFee
			);

			// Update storage
			CommunityFees::<T>::insert(
				community_id.clone(),
				FeesToPay {
					sender: sender_fee,
					receiver: receiver_fee,
				},
			);

			// Emit event
			Self::deposit_event(Event::SettedNewFees { community_id });

			Ok(().into())
		}
	}
}

impl<T: Config> FeeHandler<T> for Pallet<T> {
	fn apply_fees(
		_asset: &AssetIdOf<T>,
		sender: &T::AccountId,
		beneficiary: &T::AccountId,
		_amount: &BalanceOf<T>,
		_remark: Option<&[u8]>,
	) -> pallet_payments::Fees<T> {
		let get_community_id =
			|who| match PalletId::try_from_sub_account::<T::CommunityId>(who) {
				Some((pid, community_id))if T::CommunitiesPalletId::get() == pid => Some(community_id),
				_ => None
			};

		let sender_fees = if let Some(community_id)= get_community_id(sender) {
			Self::community_fees()
		} else {
			T::MandatoryFee::get()
		};

		let receiver_fees = if let Some(community_id)= get_community_id(beneficiary) {
			
		} else {
			T::MandatoryFee::get()
		};

		pallet_payments::Fees { sender_pays: sender_fees, beneficiary_pays: rec }
	}
}
