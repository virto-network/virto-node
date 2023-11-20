use super::*;
#[allow(unused)]
use crate::{types::*, Pallet as Payments};
use frame_benchmarking::{account, v2::*};
use frame_support::{
	traits::{
		fungibles::{Inspect, Mutate},
		Get,
	},
	BoundedVec,
};
use frame_system::Origin;
use frame_system::RawOrigin;

const SEED: u32 = 1;

// See if `generic_event` has been emitted.
fn assert_has_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

// Compare `generic_event` to the last emitted event.
fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[benchmarks(
	where
		<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId: Zero,
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn pay(q: Linear<1, { T::MaxRemarkLength::get() }>) -> Result<(), BenchmarkError> {
		let sender: T::AccountId = account("Alice", 0, 1);
		let beneficiary: T::AccountId = account("Bob", 0, 2);
		T::BenchmarkHelper::set_balance(beneficiary.clone(), <BalanceOf<T>>::from(100_000_000_u32));
		let beneficiary_lookup = T::Lookup::unlookup(beneficiary.clone());
		let asset = <AssetIdOf<T>>::zero();
		let amount = <BalanceOf<T>>::from(50_u32);

		let remark: Option<BoundedDataOf<T>> = if q == 0 {
			None
		} else {
			Some(BoundedVec::try_from(vec![1 as u8; q as usize]).unwrap())
		};

		T::BenchmarkHelper::create_asset(asset.clone(), sender.clone(), true, <BalanceOf<T>>::from(1u32));
		T::Assets::mint_into(asset.clone(), &sender, <BalanceOf<T>>::from(100000u32))?;

		#[extrinsic_call]
		_(
			RawOrigin::Signed(sender.clone()),
			beneficiary_lookup,
			asset.clone(),
			amount,
			remark.clone(),
		);

		assert_last_event::<T>(
			Event::PaymentCreated {
				sender,
				beneficiary,
				asset,
				amount,
				remark,
			}
			.into(),
		);
		Ok(())
	}

	impl_benchmark_test_suite!(Payments, crate::mock::new_test_ext(), crate::mock::Test);
}
