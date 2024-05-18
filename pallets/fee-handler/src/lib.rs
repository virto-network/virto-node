#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use pallet_payments::{AccountIdLookupOf, AssetIdOf, BalanceOf, BoundedDataOf, FeeHandler};
pub use parity_scale_codec::{Decode, Encode, MaxEncodedLen};


#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::tokens::Balance};

	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{Member,Zero};

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_payments::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type CommunityId: Member + Parameter + MaxEncodedLen;
		type Fee: Balance + Parameter + MaxEncodedLen + From<u64>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type CommunityFees<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::CommunityId,   // CommunityId
		(T::Fee, T::Fee), // Sender, Receiver
	>;

	#[pallet::event]
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
		/// Not authorized to set the fees
		UnauthorizedToSetValue,
		/// Unexpected math error
		MathError,
		/// Community does not exist
		NonExistingCommunity,
		/// Setting same value
		SettingSameValue,
	}

	#[pallet::composite_enum]
	pub enum HoldReason {
		#[codec(index = 0)]
		TransferPayment,
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
			let _who = ensure_signed(origin)?;
			// Modify storage CommunityFees
			// Validate fees
			ensure!(
				sender_fee >= Zero::zero() && receiver_fee >= Zero::zero(),
				Error::<T>::InvalidFee
			);

			// Update storage
			<CommunityFees<T>>::insert(community_id, (sender_fee, receiver_fee));

			// Emit event
			// Self::deposit_event(Event::SettedNewFees { community_id });

			Ok(().into())
		}
	}
}

impl<T: Config> FeeHandler<T> for Pallet<T> {
	fn apply_fees(
		asset: &pallet_payments::AssetIdOf<T>,
		sender: &T::AccountId,
		beneficiary: &T::AccountId,
		amount: &pallet_payments::BalanceOf<T>,
		remark: Option<&[u8]>,
	) -> pallet_payments::Fees<T> {
		todo!()
	}
}
