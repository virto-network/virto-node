use super::*;
#[allow(unused)]
use crate::{smart_logic::Rule::Promotion, smart_logic::*, types::*, Pallet as Cards};
use frame_benchmarking::{account, v2::*};
use frame_support::{
	traits::{
		fungibles::{Inspect, Mutate},
		Get, UnixTime,
	},
	BoundedBTreeSet,
};
use frame_system::RawOrigin;
use sp_arithmetic::{traits::Saturating, Percent};

const SEED: u32 = 1;

// See if `generic_event` has been emitted.
fn assert_has_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

// Compare `generic_event` to the last emitted event.
fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn generate_remark(q: u8) -> Option<Vec<u8>> {
	if q == 0 {
		None
	} else {
		Some(vec![0; q as usize])
	}
}

fn origin<T: Config>(o: Origin) -> RawOrigin<T::AccountId> {
	T::BenchmarkHelper::origin(o)
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
		let asset = <AssetIdOf<T>>::zero();
		let amount = <BalanceOf<T>>::from(50u32);
		let remark = generate_remark(q.into());
		T::BenchmarkHelper::create_asset(asset, sender, true, <BalanceOf<T>>::from(100u8));
		T::Assets::mint_into(asset, &sender, 50_000_000_u32.into())?;

		#[extrinsic_call]
		_(origin::<T>(RawOrigin::Signed(SEED)), beneficiary, asset, amount, remark);

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
