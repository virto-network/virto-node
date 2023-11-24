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

use frame_system::RawOrigin;
use sp_runtime::Percent;

// Compare `generic_event` to the last emitted event.
fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn create_accounts<T: Config>() -> (T::AccountId, T::AccountId, AccountIdLookupOf<T>, AccountIdLookupOf<T>) {
	let sender: T::AccountId = account("Alice", 0, 10);
	let beneficiary: T::AccountId = account("Bob", 0, 11);
	let sender_lookup = T::Lookup::unlookup(sender.clone());
	let beneficiary_lookup = T::Lookup::unlookup(beneficiary.clone());

	(sender, beneficiary, sender_lookup, beneficiary_lookup)
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
) -> Result<
	(
		T::PaymentId,
		T::AccountId,
		T::AccountId,
		AccountIdLookupOf<T>,
		AccountIdLookupOf<T>,
	),
	BenchmarkError,
> {
	let (sender, beneficiary, sender_lookup, beneficiary_lookup) = create_accounts::<T>();
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

	Ok((payment_id, sender, beneficiary, sender_lookup, beneficiary_lookup))
}

#[benchmarks(
	where
		<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId: Zero,
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn pay(q: Linear<1, { T::MaxRemarkLength::get() }>) -> Result<(), BenchmarkError> {
		let (sender, beneficiary, _, beneficiary_lookup) = create_accounts::<T>();
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
		let (payment_id, sender, beneficiary, _, beneficiary_lookup) = create_payment::<T>(&amount, &asset, None)?;

		#[extrinsic_call]
		_(RawOrigin::Signed(sender.clone()), beneficiary_lookup, payment_id);

		assert_last_event::<T>(Event::PaymentReleased { sender, beneficiary }.into());
		Ok(())
	}

	#[benchmark]
	fn cancel() -> Result<(), BenchmarkError> {
		let amount = <BalanceOf<T>>::from(50_u32);
		let asset = <AssetIdOf<T>>::zero();
		let (payment_id, sender, beneficiary, sender_lookup, _beneficiary_lookup) =
			create_payment::<T>(&amount, &asset, None)?;

		#[extrinsic_call]
		_(RawOrigin::Signed(beneficiary.clone()), sender_lookup, payment_id);

		assert_last_event::<T>(Event::PaymentCancelled { sender, beneficiary }.into());
		Ok(())
	}

	#[benchmark]
	fn request_refund() -> Result<(), BenchmarkError> {
		let amount = <BalanceOf<T>>::from(50_u32);
		let asset = <AssetIdOf<T>>::zero();
		let (payment_id, sender, beneficiary, _sender_lookup, beneficiary_lookup) =
			create_payment::<T>(&amount, &asset, None)?;

		#[extrinsic_call]
		_(RawOrigin::Signed(sender.clone()), beneficiary_lookup, payment_id);

		let current_block = frame_system::Pallet::<T>::block_number();
		let expiry = current_block + T::CancelBufferBlockLength::get();

		assert_last_event::<T>(
			Event::PaymentCreatorRequestedRefund {
				sender,
				beneficiary,
				expiry,
			}
			.into(),
		);
		Ok(())
	}

	#[benchmark]
	fn dispute_refund() -> Result<(), BenchmarkError> {
		let amount = <BalanceOf<T>>::from(50_u32);
		let asset = <AssetIdOf<T>>::zero();
		let (payment_id, sender, beneficiary, sender_lookup, beneficiary_lookup) =
			create_payment::<T>(&amount, &asset, None)?;

		assert!(Payments::<T>::request_refund(
			RawOrigin::Signed(sender.clone()).into(),
			beneficiary_lookup,
			payment_id
		)
		.is_ok());

		#[extrinsic_call]
		_(RawOrigin::Signed(beneficiary.clone()), sender_lookup, payment_id);

		assert_last_event::<T>(Event::PaymentRefundDisputed { sender, beneficiary }.into());
		Ok(())
	}

	#[benchmark]
	fn resolve_dispute() -> Result<(), BenchmarkError> {
		let amount = <BalanceOf<T>>::from(50_u32);
		let asset = <AssetIdOf<T>>::zero();
		let (payment_id, sender, beneficiary, sender_lookup, beneficiary_lookup) =
			create_payment::<T>(&amount, &asset, None)?;

		assert!(Payments::<T>::request_refund(
			RawOrigin::Signed(sender.clone()).into(),
			beneficiary_lookup.clone(),
			payment_id
		)
		.is_ok());

		assert!(Payments::<T>::dispute_refund(
			RawOrigin::Signed(beneficiary.clone()).into(),
			sender_lookup.clone(),
			payment_id
		)
		.is_ok());

		let dispute_result = DisputeResult {
			percent_beneficiary: Percent::from_percent(90),
			in_favor_of: Role::Sender,
		};

		#[extrinsic_call]
		_(
			RawOrigin::Root,
			sender_lookup,
			beneficiary_lookup,
			payment_id,
			dispute_result,
		);

		assert_last_event::<T>(Event::PaymentDisputeResolved { sender, beneficiary }.into());
		Ok(())
	}

	#[benchmark]
	fn request_payment() -> Result<(), BenchmarkError> {
		let (sender, beneficiary, sender_lookup, _beneficiary_lookup) = create_accounts::<T>();
		let asset: AssetIdOf<T> = <AssetIdOf<T>>::zero();
		create_and_mint_asset::<T>(&sender, &beneficiary, &asset, &<BalanceOf<T>>::from(100000u32))?;
		let amount = <BalanceOf<T>>::from(50_u32);

		#[extrinsic_call]
		_(RawOrigin::Signed(beneficiary.clone()), sender_lookup, asset, amount);

		assert_last_event::<T>(Event::PaymentRequestCreated { sender, beneficiary }.into());
		Ok(())
	}

	#[benchmark]
	fn accept_and_pay() -> Result<(), BenchmarkError> {
		let (sender, beneficiary, sender_lookup, beneficiary_lookup) = create_accounts::<T>();
		let asset: AssetIdOf<T> = <AssetIdOf<T>>::zero();
		create_and_mint_asset::<T>(&sender, &beneficiary, &asset, &<BalanceOf<T>>::from(100000u32))?;
		let amount = <BalanceOf<T>>::from(50_u32);
		let payment_id: T::PaymentId = Payments::<T>::next_payment_id()?;

		assert!(Payments::<T>::request_payment(
			RawOrigin::Signed(beneficiary.clone()).into(),
			sender_lookup,
			asset,
			amount
		)
		.is_ok());

		#[extrinsic_call]
		_(RawOrigin::Signed(sender.clone()), beneficiary_lookup, payment_id);

		assert_last_event::<T>(Event::PaymentRequestCreated { sender, beneficiary }.into());
		Ok(())
	}

	impl_benchmark_test_suite!(Payments, crate::mock::new_test_ext(), crate::mock::Test);
}
