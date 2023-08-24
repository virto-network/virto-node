use super::*;
use crate::{
	mock::*,
	types::{PaymentDetail, PaymentState},
	HoldReason, Payment as PaymentStore,
};
use frame_support::{assert_ok, traits::fungibles};

use sp_runtime::BoundedVec;

/// What we will do:
/// Sender(2) pays 20 tokens to the PAYMENT_BENEFICIARY(21)
/// Sender pays the following fees:
///   - 2 tokens to the FEE_SENDER_ACCOUNT(30)
///   - 10% of the payment amount (meaning 2 tokens) to the
///     FEE_SYSTEM_ACCOUNT(31)
///   (total of 4 tokens)
/// Beneficiary pays the following fees:
///   - 3 tokens to the FEE_BENEFICIARY_ACCOUNT(32)
///   - 10% of the payment amount (meaning 2 tokens) to the
///     FEE_SYSTEM_ACCOUNT(31)
///   (total of 5 tokens)
/// The PAYMENT_BENEFICIARY will receive 15 tokens free of charge
/// The sender will need have a balance at least of 26 tokens to make the
/// purchase:
///  - 20 tokens for the payment + 4 tokens for the fee + 2 tokens for the
///    incentive
#[test]
fn test_pay_and_release_works() {
	new_test_ext().execute_with(|| {
		let remark: BoundedVec<u8, MaxRemarkLength> = BoundedVec::truncate_from(b"remark".to_vec());
		let reason: &<Test as Config>::RuntimeHoldReason = &HoldReason::TransferPayment.into();

		assert_ok!(Payments::pay(
			RuntimeOrigin::signed(SENDER_ACCOUNT),
			PAYMENT_BENEFICIARY,
			ASSET_ID,
			PAYMENT_AMOUNT,
			Some(remark.clone()),
		));

		System::assert_has_event(
			RuntimeEvent::Payments(pallet_payments::Event::PaymentCreated {
				sender: SENDER_ACCOUNT,
				beneficiary: PAYMENT_BENEFICIARY,
				asset: ASSET_ID,
				amount: PAYMENT_AMOUNT,
				remark: Some(remark.clone()),
			})
			.into(),
		);

		let fees_details: Fees<Test> = <Test as pallet_payments::Config>::FeeHandler::apply_fees(
			&SENDER_ACCOUNT,
			&PAYMENT_BENEFICIARY,
			&PAYMENT_AMOUNT,
			Some(remark.as_slice()),
		);

		assert_eq!(
			PaymentStore::<Test>::get(SENDER_ACCOUNT, PAYMENT_BENEFICIARY),
			Some(PaymentDetail {
				asset: ASSET_ID,
				amount: PAYMENT_AMOUNT,
				incentive_amount: INCENTIVE_AMOUNT,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fees_details: fees_details.clone(),
			})
		);

		assert_eq!(
			<Assets as fungibles::InspectHold<_>>::balance_on_hold(ASSET_ID, reason, &PAYMENT_BENEFICIARY),
			PAYMENT_AMOUNT
		);
		assert_eq!(
			<Assets as fungibles::InspectHold<_>>::balance_on_hold(ASSET_ID, reason, &SENDER_ACCOUNT),
			INCENTIVE_AMOUNT + FEE_SENDER_AMOUNT + INCENTIVE_AMOUNT
		);

		assert_ok!(Payments::release(
			RuntimeOrigin::signed(SENDER_ACCOUNT),
			PAYMENT_BENEFICIARY
		));

		System::assert_has_event(
			RuntimeEvent::Payments(pallet_payments::Event::PaymentReleased {
				sender: SENDER_ACCOUNT,
				beneficiary: PAYMENT_BENEFICIARY,
			})
			.into(),
		);

		assert_eq!(
			PaymentStore::<Test>::get(SENDER_ACCOUNT, PAYMENT_BENEFICIARY),
			Some(PaymentDetail {
				asset: ASSET_ID,
				amount: PAYMENT_AMOUNT,
				incentive_amount: INCENTIVE_AMOUNT,
				state: PaymentState::Finished,
				resolver_account: RESOLVER_ACCOUNT,
				fees_details,
			})
		);

		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &FEE_SYSTEM_ACCOUNT),
			EXPECTED_SYSTEM_TOTAL_FEE
		);

		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &FEE_SENDER_ACCOUNT),
			FEE_SENDER_AMOUNT
		);
		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &FEE_BENEFICIARY_ACCOUNT),
			FEE_BENEFICIARY_AMOUNT
		);
		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &PAYMENT_BENEFICIARY),
			PAYMENT_AMOUNT - FEE_BENEFICIARY_AMOUNT - SYSTEM_FEE
		);

		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &SENDER_ACCOUNT),
			INITIAL_BALANCE - PAYMENT_AMOUNT - FEE_SENDER_AMOUNT - SYSTEM_FEE
		);
	});
}

#[test]
fn test_pay_and_cancel_works() {
	new_test_ext().execute_with(|| {
		let remark: BoundedVec<u8, MaxRemarkLength> = BoundedVec::truncate_from(b"remark".to_vec());
		let reason: &<Test as Config>::RuntimeHoldReason = &HoldReason::TransferPayment.into();

		assert_ok!(Payments::pay(
			RuntimeOrigin::signed(SENDER_ACCOUNT),
			PAYMENT_BENEFICIARY,
			ASSET_ID,
			PAYMENT_AMOUNT,
			Some(remark.clone()),
		));

		System::assert_has_event(
			RuntimeEvent::Payments(pallet_payments::Event::PaymentCreated {
				sender: SENDER_ACCOUNT,
				beneficiary: PAYMENT_BENEFICIARY,
				asset: ASSET_ID,
				amount: PAYMENT_AMOUNT,
				remark: Some(remark.clone()),
			})
			.into(),
		);

		let fees_details: Fees<Test> = <Test as pallet_payments::Config>::FeeHandler::apply_fees(
			&SENDER_ACCOUNT,
			&PAYMENT_BENEFICIARY,
			&PAYMENT_AMOUNT,
			Some(remark.as_slice()),
		);

		assert_eq!(
			PaymentStore::<Test>::get(SENDER_ACCOUNT, PAYMENT_BENEFICIARY),
			Some(PaymentDetail {
				asset: ASSET_ID,
				amount: PAYMENT_AMOUNT,
				incentive_amount: INCENTIVE_AMOUNT,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fees_details: fees_details.clone(),
			})
		);

		assert_eq!(
			<Assets as fungibles::InspectHold<_>>::balance_on_hold(ASSET_ID, reason, &PAYMENT_BENEFICIARY),
			PAYMENT_AMOUNT
		);
		assert_eq!(
			<Assets as fungibles::InspectHold<_>>::balance_on_hold(ASSET_ID, reason, &SENDER_ACCOUNT),
			INCENTIVE_AMOUNT + FEE_SENDER_AMOUNT + INCENTIVE_AMOUNT
		);

		assert_ok!(Payments::cancel(
			RuntimeOrigin::signed(PAYMENT_BENEFICIARY),
			SENDER_ACCOUNT,
		));

		System::assert_has_event(
			RuntimeEvent::Payments(pallet_payments::Event::PaymentCancelled {
				sender: SENDER_ACCOUNT,
				beneficiary: PAYMENT_BENEFICIARY,
			})
			.into(),
		);

		assert_eq!(PaymentStore::<Test>::get(SENDER_ACCOUNT, PAYMENT_BENEFICIARY), None);

		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &FEE_SYSTEM_ACCOUNT),
			0
		);

		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &FEE_SENDER_ACCOUNT),
			0
		);
		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &FEE_BENEFICIARY_ACCOUNT),
			0
		);
		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &PAYMENT_BENEFICIARY),
			0
		);

		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &SENDER_ACCOUNT),
			100
		);
	});
}
