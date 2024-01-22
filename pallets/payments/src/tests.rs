use super::*;
use crate::{
	mock::*,
	types::{PaymentDetail, PaymentState},
	LastId, Payment as PaymentStore,
};
use frame_support::{assert_ok, traits::fungibles, weights::constants::WEIGHT_REF_TIME_PER_NANOS};
use weights::SubstrateWeight;

use sp_runtime::{BoundedVec, Perbill};

fn build_payment(assertion: bool) -> Fees<Test> {
	let remark: BoundedVec<u8, MaxRemarkLength> = BoundedVec::truncate_from(b"remark".to_vec());
	let reason: &<Test as Config>::RuntimeHoldReason = &HoldReason::TransferPayment.into();

	assert_ok!(Payments::pay(
		RuntimeOrigin::signed(SENDER_ACCOUNT),
		PAYMENT_BENEFICIARY,
		ASSET_ID,
		PAYMENT_AMOUNT,
		Some(remark.clone()),
	));

	let fees_details: Fees<Test> = <Test as pallet_payments::Config>::FeeHandler::apply_fees(
		&ASSET_ID,
		&SENDER_ACCOUNT,
		&PAYMENT_BENEFICIARY,
		&PAYMENT_AMOUNT,
		Some(remark.as_slice()),
	);

	if assertion == true {
		System::assert_has_event(RuntimeEvent::Payments(pallet_payments::Event::PaymentCreated {
			sender: SENDER_ACCOUNT,
			beneficiary: PAYMENT_BENEFICIARY,
			asset: ASSET_ID,
			amount: PAYMENT_AMOUNT,
			remark: Some(remark.clone()),
		}));

		assert_eq!(
			PaymentStore::<Test>::get((SENDER_ACCOUNT, PAYMENT_BENEFICIARY, PAYMENT_ID)).unwrap(),
			PaymentDetail {
				asset: ASSET_ID,
				amount: PAYMENT_AMOUNT,
				incentive_amount: INCENTIVE_AMOUNT,
				state: PaymentState::Created,
				fees: fees_details.clone(),
			}
		);

		assert_eq!(
			<Assets as fungibles::InspectHold<_>>::balance_on_hold(ASSET_ID, reason, &PAYMENT_BENEFICIARY),
			PAYMENT_AMOUNT
		);
		assert_eq!(
			<Assets as fungibles::InspectHold<_>>::balance_on_hold(ASSET_ID, reason, &SENDER_ACCOUNT),
			INCENTIVE_AMOUNT + FEE_SENDER_AMOUNT + EXPECTED_SYSTEM_SENDER_FEE
		);
	}

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
/// Sender(2) pays 20 tokens to the PAYMENT_BENEFICIARY(11)
/// Sender pays the following fees:
///   - 2 tokens to the FEE_SENDER_ACCOUNT(30)
///   - 15% of the payment amount (meaning 3 tokens) to the
///     FEE_SYSTEM_ACCOUNT(31)
///   (total of 5 tokens)
/// Beneficiary pays the following fees:
///   - 3 tokens to the FEE_BENEFICIARY_ACCOUNT(32)
///   - 15% of the payment amount (meaning 3 tokens) to the
///     FEE_SYSTEM_ACCOUNT(31)
///   (total of 6 tokens)
/// The PAYMENT_BENEFICIARY will receive 14 tokens free of charge
/// The sender will need have a balance at least of 27 tokens to make the
/// purchase:
///  - 20 tokens for the payment + 5 tokens for the fee + 2 tokens for the
///    incentive
#[test]
fn test_pay_and_release_works() {
	new_test_ext().execute_with(|| {
		let fees: Fees<Test> = build_payment(true);

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
				fees,
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
		build_payment(true);
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
		let fees: Fees<Test> = build_payment(true);

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
				fees,
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
///           - 15% of the payment amount (meaning 3 tokens) to the
///             FEE_SYSTEM_ACCOUNT
///        The PAYMENT_BENEFICIARY should pay the mandatory fee:
///           - 15% of the payment amount (meaning 3 tokens) to the
///             FEE_SYSTEM_ACCOUNT
///
///     Fee not deducted during dispute:
///        SENDER's fee:
///           - 2 tokens to the FEE_SENDER_AMOUNT
///        PAYMENT_BENEFICIARY's fee:
///           - 3 tokens to FEE_BENEFICIARY_AMOUNT
///
///  4.1) PAYMENT_BENEFICIARY should receive:
///    + 18 token (90% because of dispute ruling)
///    + 2 token (incentive amount give back because of wining side of dispute)
///    - 3 tokens (deducted mandatory fee)
///  Beneficiary receives: 17 token / total balance: 27 token.
///  4.2) SENDER should receive:
///    + 2 token (remaining 10% of dispute ruling)
///    + 2 token (fee not deducted )
///    - 2 token (incentive amount deducted because of wining side of dispute)
///    - 3 tokens (deducted mandatory fee)
///    total: -1 token / total balance: 73 token.
///
///  Sender's expected Balance after dispute:
///    100(initial) - 18(payment) - 3(system fee) - 2(incentive for loosing) =
/// 77  Beneficiary's expected Balance after dispute:
///    10(initial) + 18(payment) - 3(system fee) = 25

#[test]
fn payment_disputed_beneficiary_wins() {
	new_test_ext().execute_with(|| {
		const EXPECTED_BALANCE_SENDER: u64 = 77;
		const EXPECTED_BALANCE_BENEFICIARY: u64 = 25;

		let _ = Assets::mint(
			RuntimeOrigin::signed(ASSET_ADMIN_ACCOUNT),
			ASSET_ID,
			PAYMENT_BENEFICIARY,
			10,
		);

		let fees: Fees<Test> = build_payment(true);

		assert_ok!(Payments::request_refund(
			RuntimeOrigin::signed(SENDER_ACCOUNT),
			PAYMENT_BENEFICIARY,
			PAYMENT_ID
		));

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
				fees,
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

		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &SENDER_ACCOUNT),
			EXPECTED_BALANCE_SENDER
		);

		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &PAYMENT_BENEFICIARY),
			EXPECTED_BALANCE_BENEFICIARY
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
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &FEE_SYSTEM_ACCOUNT),
			EXPECTED_SYSTEM_TOTAL_FEE
		);
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
/// 4) The RESOLVER, rule in favor of SENDER to pay 90%.
///
///     Mandatory Fees:
///        The SENDER should pay the mandatory fee:
///           - 15% of the payment amount (meaning 3 tokens) to the
///             FEE_SYSTEM_ACCOUNT
///        The PAYMENT_BENEFICIARY should pay the mandatory fee:
///           - 15% of the payment amount (meaning 3 tokens) to the
///             FEE_SYSTEM_ACCOUNT
///
///     Fee not deducted during dispute:
///        SENDER's fee:
///           - 2 tokens to the FEE_SENDER_AMOUNT
///        PAYMENT_BENEFICIARY's fee:
///           - 3 tokens to FEE_BENEFICIARY_AMOUNT
///
///  4.1) PAYMENT_BENEFICIARY should receive:
///    + 2 token (10% because of dispute ruling)
///    - 2 token (incentive amount give back because of loosing side of dispute)
///    - 3 tokens (deducted mandatory fee)
///    Beneficiary loose: -3 tokens
///  4.2) SENDER should receive:
///     + 18 token (remaining 90% of dispute ruling)
///     + 2 token (fee not deducted )
///     + 2 token (incentive amount returned because of wining side of dispute)
///     - 3 tokens (deducted mandatory fee)
///     total: 19 token / total balance: 81 token.
///
///  Sender's expected Balance after dispute:
///    100(initial) - 2 (dispute ruling) - 3(system fee) = 95
///  Beneficiary's expected Balance after dispute:
///    10(initial) + 2(dispute ruling) - 3(system fee) - 2(incentive for
/// loosing)= 7
#[test]
fn payment_disputed_sender_wins() {
	new_test_ext().execute_with(|| {
		const EXPECTED_BALANCE_SENDER: u64 = 95;
		const EXPECTED_BALANCE_BENEFICIARY: u64 = 7;
		const EXPECTED_RESOLVER_BALANCE: u64 = 2;

		let _ = Assets::mint(
			RuntimeOrigin::signed(ASSET_ADMIN_ACCOUNT),
			ASSET_ID,
			PAYMENT_BENEFICIARY,
			10,
		);

		let fees: Fees<Test> = build_payment(true);

		assert_ok!(Payments::request_refund(
			RuntimeOrigin::signed(SENDER_ACCOUNT),
			PAYMENT_BENEFICIARY,
			PAYMENT_ID
		));

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
				fees,
			}
		);

		assert_ok!(Payments::resolve_dispute(
			RuntimeOrigin::root(),
			SENDER_ACCOUNT,
			PAYMENT_BENEFICIARY,
			PAYMENT_ID,
			DisputeResult {
				percent_beneficiary: Percent::from_percent(90),
				in_favor_of: Role::Sender
			}
		));

		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &SENDER_ACCOUNT),
			EXPECTED_BALANCE_SENDER
		);

		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &PAYMENT_BENEFICIARY),
			EXPECTED_BALANCE_BENEFICIARY
		);

		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &ROOT_ACCOUNT),
			EXPECTED_RESOLVER_BALANCE
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
			<Assets as fungibles::Inspect<_>>::balance(ASSET_ID, &FEE_SYSTEM_ACCOUNT),
			EXPECTED_SYSTEM_TOTAL_FEE
		);
	})
}

#[test]
fn request_payment() {
	new_test_ext().execute_with(|| {
		assert_ok!(Payments::request_payment(
			RuntimeOrigin::signed(PAYMENT_BENEFICIARY),
			SENDER_ACCOUNT,
			ASSET_ID,
			PAYMENT_AMOUNT
		));

		System::assert_has_event(RuntimeEvent::Payments(pallet_payments::Event::PaymentRequestCreated {
			sender: SENDER_ACCOUNT,
			beneficiary: PAYMENT_BENEFICIARY,
		}));

		let fees: Fees<Test> = <Test as pallet_payments::Config>::FeeHandler::apply_fees(
			&ASSET_ID,
			&SENDER_ACCOUNT,
			&PAYMENT_BENEFICIARY,
			&PAYMENT_AMOUNT,
			None,
		);

		assert_eq!(
			PaymentStore::<Test>::get((SENDER_ACCOUNT, PAYMENT_BENEFICIARY, PAYMENT_ID)).unwrap(),
			PaymentDetail {
				asset: ASSET_ID,
				amount: PAYMENT_AMOUNT,
				incentive_amount: INCENTIVE_AMOUNT,
				state: PaymentState::PaymentRequested,
				fees,
			}
		);

		assert_ok!(Payments::accept_and_pay(
			RuntimeOrigin::signed(SENDER_ACCOUNT),
			PAYMENT_BENEFICIARY,
			PAYMENT_ID
		));

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
	})
}

#[test]
fn next_id_works() {
	new_test_ext().execute_with(|| {
		build_payment(false);

		assert_eq!(LastId::<Test>::get().unwrap(), 1);
		build_payment(false);
		assert_eq!(LastId::<Test>::get().unwrap(), 2);

		assert_ok!(Payments::request_payment(
			RuntimeOrigin::signed(PAYMENT_BENEFICIARY),
			SENDER_ACCOUNT,
			ASSET_ID,
			PAYMENT_AMOUNT
		));

		assert_eq!(LastId::<Test>::get().unwrap(), 3);
	});
}

#[test]
fn weights() {
	use crate::weights::WeightInfo;
	use frame_support::weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight};
	// max block: 0.5s compute with 12s average block time
	const MAX_BLOCK_REF_TIME: u64 = WEIGHT_REF_TIME_PER_SECOND.saturating_div(2); // https://github.com/paritytech/cumulus/blob/98e68bd54257b4039a5d5b734816f4a1b7c83a9d/parachain-template/runtime/src/lib.rs#L221
	const MAX_BLOCK_POV_SIZE: u64 = 5 * 1024 * 1024; // https://github.com/paritytech/polkadot/blob/ba1f65493d91d4ab1787af2fd6fe880f1da90586/primitives/src/v4/mod.rs#L384
	const MAX_BLOCK_WEIGHT: Weight = Weight::from_parts(MAX_BLOCK_REF_TIME, MAX_BLOCK_POV_SIZE);
	// max extrinsics: 75% of block
	const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75); // https://github.com/paritytech/cumulus/blob/d20c4283fe85df0c1ef8cb7c9eb7c09abbcbfa31/parachain-template/runtime/src/lib.rs#L218
	let max_total_extrinsics = MAX_BLOCK_WEIGHT * NORMAL_DISPATCH_RATIO;
	// max extrinsic: max total extrinsics less average on_initialize ratio and less
	// base extrinsic weight
	const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5); // https://github.com/paritytech/cumulus/blob/d20c4283fe85df0c1ef8cb7c9eb7c09abbcbfa31/parachain-template/runtime/src/lib.rs#L214
	const BASE_EXTRINSIC: Weight = Weight::from_parts(WEIGHT_REF_TIME_PER_NANOS.saturating_mul(125_000), 0); // https://github.com/paritytech/cumulus/blob/d20c4283fe85df0c1ef8cb7c9eb7c09abbcbfa31/parachain-template/runtime/src/weights/extrinsic_weights.rs#L26
	let max_extrinsic_weight = max_total_extrinsics
		.saturating_sub(MAX_BLOCK_WEIGHT * AVERAGE_ON_INITIALIZE_RATIO)
		.saturating_sub(BASE_EXTRINSIC);
	assert_eq!(max_extrinsic_weight, Weight::from_parts(349_875_000_000, 3_670_016));

	println!("max block weight: {MAX_BLOCK_WEIGHT}");
	println!("max total extrinsics weight: {max_total_extrinsics}");
	println!("max extrinsic weight: {max_extrinsic_weight}\n");

	let mut total = Weight::zero();
	for (function, weight) in vec![
		// Examples: call available weight functions with various parameters (as applicable) to gauge weight usage in
		// comparison to limits
		("pay (20)", SubstrateWeight::<Test>::pay(20_u32)),
		("release", SubstrateWeight::<Test>::release()),
		("cancel", SubstrateWeight::<Test>::cancel()),
		("request_refund", SubstrateWeight::<Test>::request_refund()),
		("dispute_refund", SubstrateWeight::<Test>::dispute_refund()),
		("resolve_dispute", SubstrateWeight::<Test>::resolve_dispute()),
		("request_payment", SubstrateWeight::<Test>::request_payment()),
		("accept_and_pay", SubstrateWeight::<Test>::accept_and_pay()),
	] {
		println!("{function}: {weight:?}",);
		println!(
			"\tpercentage of max extrinsic weight: {:.2}% (ref_time), {:.2}% (proof_size)",
			(weight.ref_time() as f64 / max_extrinsic_weight.ref_time() as f64) * 100.0,
			(weight.proof_size() as f64 / max_extrinsic_weight.proof_size() as f64) * 100.0,
		);
		println!(
			"\tmax tx per block: {} (ref_time), {} (proof_size)",
			max_extrinsic_weight.ref_time() / weight.ref_time(),
			max_extrinsic_weight.proof_size() / weight.proof_size()
		);
		assert!(weight.all_lt(max_extrinsic_weight));

		total += weight;
	}

	// output total weight, useful for evaluating net weight changes when optimising
	println!("\ntotal weight: {total:?}");
}
