use super::*;
use crate::{
	mock::*,
	types::{PaymentDetail, PaymentState},
	HoldReason, Payment as PaymentStore,
};
use frame_support::{assert_ok, traits::fungibles};

use sp_runtime::BoundedVec;

fn build_payment() -> Fees<Test> {
	let remark: BoundedVec<u8, MaxRemarkLength> = BoundedVec::truncate_from(b"remark".to_vec());
	let reason: &<Test as Config>::RuntimeHoldReason = &HoldReason::TransferPayment.into();

	assert_ok!(Payments::pay(
		RuntimeOrigin::signed(SENDER_ACCOUNT),
		PAYMENT_BENEFICIARY,
		ASSET_ID,
		PAYMENT_AMOUNT,
		Some(remark.clone()),
	));

	System::assert_has_event(RuntimeEvent::Payments(pallet_payments::Event::PaymentCreated {
		sender: SENDER_ACCOUNT,
		beneficiary: PAYMENT_BENEFICIARY,
		asset: ASSET_ID,
		amount: PAYMENT_AMOUNT,
		remark: Some(remark.clone()),
	}));

	let fees_details: Fees<Test> = <Test as pallet_payments::Config>::FeeHandler::apply_fees(
		&ASSET_ID,
		&SENDER_ACCOUNT,
		&PAYMENT_BENEFICIARY,
		&PAYMENT_AMOUNT,
		Some(remark.as_slice()),
	);

	assert_eq!(
		PaymentStore::<Test>::get((SENDER_ACCOUNT, PAYMENT_BENEFICIARY, PAYMENT_ID)).unwrap(),
		PaymentDetail {
			asset: ASSET_ID,
			amount: PAYMENT_AMOUNT,
			incentive_amount: INCENTIVE_AMOUNT,
			state: PaymentState::Created,
			fees_details: fees_details.clone(),
		}
	);

	assert_eq!(
		<Assets as fungibles::InspectHold<_>>::balance_on_hold(ASSET_ID, reason, &PAYMENT_BENEFICIARY),
		PAYMENT_AMOUNT
	);
	assert_eq!(
		<Assets as fungibles::InspectHold<_>>::balance_on_hold(ASSET_ID, reason, &SENDER_ACCOUNT),
		INCENTIVE_AMOUNT + FEE_SENDER_AMOUNT + INCENTIVE_AMOUNT
	);

	fees_details
}

fn check_balance_cancellation() {
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
}

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
		let fees_details: Fees<Test> = build_payment();

		assert_ok!(Payments::release(
			RuntimeOrigin::signed(SENDER_ACCOUNT),
			PAYMENT_BENEFICIARY,
			PAYMENT_ID
		));

		System::assert_has_event(RuntimeEvent::Payments(pallet_payments::Event::PaymentReleased {
			sender: SENDER_ACCOUNT,
			beneficiary: PAYMENT_BENEFICIARY,
		}));

		assert_eq!(
			PaymentStore::<Test>::get((SENDER_ACCOUNT, PAYMENT_BENEFICIARY, PAYMENT_ID)).unwrap(),
			PaymentDetail {
				asset: ASSET_ID,
				amount: PAYMENT_AMOUNT,
				incentive_amount: INCENTIVE_AMOUNT,
				state: PaymentState::Finished,
				fees_details,
			}
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
		build_payment();
		assert_ok!(Payments::cancel(
			RuntimeOrigin::signed(PAYMENT_BENEFICIARY),
			SENDER_ACCOUNT,
			PAYMENT_ID
		));

		System::assert_has_event(RuntimeEvent::Payments(pallet_payments::Event::PaymentCancelled {
			sender: SENDER_ACCOUNT,
			beneficiary: PAYMENT_BENEFICIARY,
		}));

		// This validates that the payment was removed from the storage.
		assert!(PaymentStore::<Test>::get((SENDER_ACCOUNT, PAYMENT_BENEFICIARY, PAYMENT_ID)).is_err());

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

#[test]
fn payment_refunded_request() {
	new_test_ext().execute_with(|| {
		let fees_details: Fees<Test> = build_payment();

		assert_ok!(Payments::request_refund(
			RuntimeOrigin::signed(SENDER_ACCOUNT),
			PAYMENT_BENEFICIARY,
			PAYMENT_ID
		));

		assert_eq!(
			PaymentStore::<Test>::get((SENDER_ACCOUNT, PAYMENT_BENEFICIARY, PAYMENT_ID)).unwrap(),
			PaymentDetail {
				asset: ASSET_ID,
				amount: PAYMENT_AMOUNT,
				incentive_amount: INCENTIVE_AMOUNT,
				state: PaymentState::RefundRequested { cancel_block: 11 },
				fees_details,
			}
		);

		run_to_block(11);

		System::assert_has_event(RuntimeEvent::Payments(pallet_payments::Event::PaymentRefunded {
			sender: SENDER_ACCOUNT,
			beneficiary: PAYMENT_BENEFICIARY,
		}));

		check_balance_cancellation();
	})
}

/// Initial balances before transactions:
/// SENDER(10) = 100 / PAYMENT_BENEFICIARY(11) = 10
///
/// Testing scenario:
/// 1) SENDER pays 20 tokens to the PAYMENT_BENEFICIARY
/// 2) Sender requests a refund.
/// 3) The PAYMENT_BENEFICIARY disputes the refund, a total of 2 tokens are
/// locked from the PAYMENT_BENEFICIARY because of the incentive amount.
/// 4) The RESOLVER, rule in favor of PAYMENT_BENEFICIARY to pay 90%.
///
///     Mandatory Fees:
///        The SENDER should pay the mandatory fee:
///           - 10% of the payment amount (meaning 2 tokens) to the
///             FEE_SYSTEM_ACCOUNT
///        The PAYMENT_BENEFICIARY should pay the mandatory fee:
///           - 10% of the payment amount (meaning 2 tokens) to the
///             FEE_SYSTEM_ACCOUNT
///
///     Fee not deducted during dispute:
///        SENDER's fee:
///           - 2 tokens to the FEE_SENDER_AMOUNT
///        PAYMENT_BENEFICIARY's fee:
///           - 3 tokens to FEE_BENEFICIARY_AMOUNT
///
///  4.1) PAYMENT_BENEFICIARY should receive:
///     + 18 token (90% because of dispute ruling)
/// 	+ 2 token (incentive amount give back because of wining side of dispute)
/// 	- 2 tokens (deducted mandatory fee)
/// 	total: 18 token / total balance: 28 token.
///  4.2) SENDER should receive:
///     + 2 token (remaining 10% of dispute ruling)
///     + 2 token (fee not deducted )
///     - 2 token (incentive amount deducted because of wining side of dispute)
///     - 2 tokens (deducted mandatory fee)
///     total: 0 token / total balance: 74 token.
///
/// The sender will need have a balance at least of 26 tokens to make the
/// purchase:
///  - 20 tokens for the payment + 4 tokens for the fee + 2 tokens for the
///    incentive
#[test]
fn payment_refunded_request_gets_disputed() {
	new_test_ext().execute_with(|| {
		let fees_details: Fees<Test> = build_payment();
		let reason: &<Test as Config>::RuntimeHoldReason = &HoldReason::TransferPayment.into();

		assert_ok!(Payments::request_refund(
			RuntimeOrigin::signed(SENDER_ACCOUNT),
			PAYMENT_BENEFICIARY,
			PAYMENT_ID
		));

		let _ = Assets::mint(
			RuntimeOrigin::signed(ASSET_ADMIN_ACCOUNT),
			ASSET_ID,
			PAYMENT_BENEFICIARY,
			10,
		);

		assert_ok!(Payments::dispute_refund(
			RuntimeOrigin::signed(PAYMENT_BENEFICIARY),
			SENDER_ACCOUNT,
			PAYMENT_ID
		));

		assert_eq!(
			PaymentStore::<Test>::get((SENDER_ACCOUNT, PAYMENT_BENEFICIARY, PAYMENT_ID)).unwrap(),
			PaymentDetail {
				asset: ASSET_ID,
				amount: PAYMENT_AMOUNT,
				incentive_amount: INCENTIVE_AMOUNT,
				state: PaymentState::NeedsReview,
				fees_details,
			}
		);

		assert_ok!(Payments::resolve_dispute(
			RuntimeOrigin::root(),
			SENDER_ACCOUNT,
			PAYMENT_BENEFICIARY,
			PAYMENT_ID,
			DisputeResult {
				percent_beneficiary: Percent::from_percent(90),
				in_favor_of: Role::Beneficiary
			}
		));

		println!(
			"FEE_SYSTEM_ACCOUNT balance {:?}",
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &FEE_SYSTEM_ACCOUNT)
		);
		println!(
			"FEE_SENDER_ACCOUNT balance {:?}",
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &FEE_SENDER_ACCOUNT)
		);
		println!(
			"FEE_BENEFICIARY_ACCOUNT balance {:?}",
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &FEE_BENEFICIARY_ACCOUNT)
		);
		println!(
			"SENDER_ACCOUNT balance {:?}",
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &SENDER_ACCOUNT)
		);
		println!(
			"PAYMENT_BENEFICIARY balance {:?}",
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &PAYMENT_BENEFICIARY)
		);

		println!(
			"HOLD SENDER_ACCOUNT balance {:?}",
			<Assets as fungibles::InspectHold<_>>::balance_on_hold(ASSET_ID, reason, &SENDER_ACCOUNT)
		);

		println!(
			"HOLD PAYMENT_BENEFICIARY balance {:?}",
			<Assets as fungibles::InspectHold<_>>::balance_on_hold(ASSET_ID, reason, &PAYMENT_BENEFICIARY)
		);
	})
}
