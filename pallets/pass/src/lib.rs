#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::large_enum_variant)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	type CallOf<T> = <T as Config>::RuntimeCall;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeCall: sp_runtime::traits::Dispatchable + Parameter;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	pub type Foo<T: Config> = StorageMap<_, Blake2_128Concat, u32, ()>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Foo,
	}

	#[pallet::error]
	pub enum Error<T> {
		Foo,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::call())]
		pub fn call(origin: OriginFor<T>, _call: Box<CallOf<T>>, _signature: [u8; 32]) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			Self::deposit_event(Event::Foo);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {}
}
