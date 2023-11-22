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

fn create_accounts<T: Config>() -> (T::AccountId, T::AccountId, AccountIdLookupOf<T>) {
	let sender: T::AccountId = account("Alice", 0, 1);
	let beneficiary: T::AccountId = account("Bob", 0, 2);
	let beneficiary_lookup = T::Lookup::unlookup(beneficiary.clone());
	(sender, beneficiary, beneficiary_lookup)
}

fn create_and_mint_asset<T: Config>(
	sender: &T::AccountId,
	beneficiary: &T::AccountId,
	asset: &AssetIdOf<T>,
	amount: &BalanceOf<T>,
) -> Result<(), BenchmarkError> {
	T::BenchmarkHelper::create_asset(asset.clone(), sender.clone(), true, <BalanceOf<T>>::from(1u32));
	T::Assets::mint_into(asset.clone(), &sender, <BalanceOf<T>>::from(100000u32))?;
	T::Assets::mint_into(asset.clone(), &beneficiary, <BalanceOf<T>>::from(100000u32))?;

	Ok(())
}

fn create_payment<T: Config>(
	amount: &BalanceOf<T>,
	asset: &AssetIdOf<T>,
	remark: Option<BoundedDataOf<T>>,
) -> Result<(T::PaymentId, T::AccountId, T::AccountId, AccountIdLookupOf<T>), BenchmarkError> {
	let (sender, beneficiary, beneficiary_lookup) = create_accounts::<T>();
	create_and_mint_asset::<T>(&sender, &beneficiary, &asset, &<BalanceOf<T>>::from(100000u32))?;

	let payment_id: T::PaymentId = Payments::<T>::next_payment_id()?;

	let payment_detail = Payments::<T>::create_payment(
		&sender,
		&beneficiary,
		asset.clone(),
		amount.clone(),
		PaymentState::Created,
		T::IncentivePercentage::get(),
		remark.as_ref().map(|x| x.as_slice()),
	)?;

	// reserve funds for payment
	Payments::<T>::reserve_payment_amount(&sender, &beneficiary, payment_detail)?;

	// TODO: check storage items

	Ok((payment_id, sender, beneficiary, beneficiary_lookup))
}

#[benchmarks(
	where
		<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId: Zero,
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn pay(q: Linear<1, { T::MaxRemarkLength::get() }>) -> Result<(), BenchmarkError> {
		let (sender, beneficiary, beneficiary_lookup) = create_accounts::<T>();
		let asset: AssetIdOf<T> = <AssetIdOf<T>>::zero();
		create_and_mint_asset::<T>(&sender, &beneficiary, &asset, &<BalanceOf<T>>::from(100000u32))?;
		let amount = <BalanceOf<T>>::from(50_u32);

		let remark: Option<BoundedDataOf<T>> = if q == 0 {
			None
		} else {
			Some(BoundedVec::try_from(vec![1 as u8; q as usize]).unwrap())
		};

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

	#[benchmark]
	fn release() -> Result<(), BenchmarkError> {
		let amount = <BalanceOf<T>>::from(50_u32);
		let asset = <AssetIdOf<T>>::zero();
		let (payment_id, sender, beneficiary, beneficiary_lookup) = create_payment::<T>(&amount, &asset, None)?;

		#[extrinsic_call]
		_(RawOrigin::Signed(sender.clone()), beneficiary_lookup, payment_id);

		assert_last_event::<T>(Event::PaymentReleased { sender, beneficiary }.into());
		Ok(())
	}

	impl_benchmark_test_suite!(Payments, crate::mock::new_test_ext(), crate::mock::Test);
}
