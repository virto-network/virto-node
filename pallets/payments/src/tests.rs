use crate::{
	mock::*,
	types::{PaymentDetail, PaymentState},
	weights::WeightInfo,
	Payment as PaymentStore, ScheduledTask, Task,
};
use frame_support::{
	assert_ok,
	traits::{fungibles, Currency},
};
use frame_system::RawOrigin;
use sp_runtime::BoundedVec;

/// What we will do:
/// Sender(2) pays 20 tokens to the beneficiary(21)
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
/// The beneficiary will receive 15 tokens free of charge
/// The sender will need have a balance at least of 26 tokens to make the
/// purchase:
///  - 20 tokens for the payment + 4 tokens for the fee + 2 tokens for the
///    incentive
#[test]
fn test_pay_and_release_works() {
	new_test_ext().execute_with(|| {
		let asset = 0;
		let admin = 1;
		let sender = 2; // account with own deposit
		let beneficiary = 21; // account with own deposit
		const INITIAL_BALANCE: u64 = 100;
		const PAYMENT_AMOUNT: u64 = 20;
		const INCENTIVE_AMOUNT: u64 = PAYMENT_AMOUNT / INCENTIVE_PERCENTAGE as u64;
		const SYSTEM_FEE: u64 = 2;
		const EXPECTED_SYSTEM_TOTAL_FEE: u64 = 4;
		Balances::make_free_balance_be(&FEE_SENDER_ACCOUNT, 100);
		Balances::make_free_balance_be(&FEE_BENEFICIARY_ACCOUNT, 100);
		Balances::make_free_balance_be(&FEE_SYSTEM_ACCOUNT, 100);
		Balances::make_free_balance_be(&beneficiary, 100);

		assert_ok!(Assets::force_create(RuntimeOrigin::root(), asset, admin, true, 1));
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(admin),
			asset,
			sender,
			INITIAL_BALANCE
		));

		let remark: BoundedVec<u8, MaxRemarkLength> = BoundedVec::truncate_from(b"remark".to_vec());

		assert_ok!(Payments::pay(
			RuntimeOrigin::signed(sender),
			beneficiary,
			asset,
			PAYMENT_AMOUNT,
			Some(remark),
			HoldIdentifiers::TransferPayment,
		));

		assert_eq!(
			<Assets as fungibles::InspectHold<_>>::balance_on_hold(
				asset,
				&HoldIdentifiers::TransferPayment,
				&beneficiary
			),
			PAYMENT_AMOUNT
		);
		assert_eq!(
			<Assets as fungibles::InspectHold<_>>::balance_on_hold(asset, &HoldIdentifiers::TransferPayment, &sender),
			INCENTIVE_AMOUNT
		);
		assert_eq!(
			<Assets as fungibles::InspectHold<_>>::balance_on_hold(
				asset,
				&HoldIdentifiers::TransferPayment,
				&FEE_SENDER_ACCOUNT
			),
			FEE_SENDER_AMOUNT
		);
		assert_eq!(
			<Assets as fungibles::InspectHold<_>>::balance_on_hold(
				asset,
				&HoldIdentifiers::TransferPayment,
				&FEE_SYSTEM_ACCOUNT
			),
			SYSTEM_FEE
		);

		assert_ok!(Payments::release(
			RuntimeOrigin::signed(sender),
			beneficiary,
			HoldIdentifiers::TransferPayment,
		));

		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(asset, &FEE_SYSTEM_ACCOUNT),
			EXPECTED_SYSTEM_TOTAL_FEE
		);

		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(asset, &FEE_SENDER_ACCOUNT),
			FEE_SENDER_AMOUNT
		);
		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(asset, &FEE_BENEFICIARY_ACCOUNT),
			FEE_BENEFICIARY_AMOUNT
		);
		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(asset, &beneficiary),
			PAYMENT_AMOUNT - FEE_BENEFICIARY_AMOUNT - SYSTEM_FEE
		);

		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(asset, &sender),
			INITIAL_BALANCE - PAYMENT_AMOUNT - FEE_SENDER_AMOUNT - SYSTEM_FEE
		);

		/*
		assert_eq!(<Assets as fungibles::Inspect<_>>::balance(asset, &dest), 16);
		let expected_sender = creator_initial_balance - payment_amount - fee + expected_incentive_amount;
		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(asset, &sender),
			expected_sender
		); */
	});
}
/*
#[test]
fn test_pay_works() {
	new_test_ext().execute_with(|| {
		let creator_initial_balance = 100;
		let payment_amount = 20;
		let expected_incentive_amount = payment_amount / INCENTIVE_PERCENTAGE as u128;

		// the payment amount should not be reserved
		assert_eq!(
			Assets::free_balance(CURRENCY_ID, &PAYMENT_CREATOR),
			creator_initial_balance
		);
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_RECIPENT), 0);

		// should be able to create a payment with available balance
		assert_ok!(Payment::pay(
			RuntimeOrigin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			CURRENCY_ID,
			payment_amount,
			None
		));
		assert_eq!(
			last_event(),
			crate::Event::<Test>::PaymentCreated {
				from: PAYMENT_CREATOR,
				asset: CURRENCY_ID,
				amount: payment_amount,
				remark: None
			}
			.into()
		);

		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				asset: CURRENCY_ID,
				amount: payment_amount,
				incentive_amount: expected_incentive_amount,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 0)),
			})
		);
		// the payment amount should be reserved correctly
		// the amount + incentive should be removed from the sender account
		assert_eq!(
			Tokens::free_balance(CURRENCY_ID, &PAYMENT_CREATOR),
			creator_initial_balance - payment_amount - expected_incentive_amount
		);
		// the incentive amount should be reserved in the sender account
		assert_eq!(
			Tokens::total_balance(CURRENCY_ID, &PAYMENT_CREATOR),
			creator_initial_balance - payment_amount
		);
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_RECIPENT), 0);
		// the transferred amount should be reserved in the recipent account
		assert_eq!(Tokens::total_balance(CURRENCY_ID, &PAYMENT_RECIPENT), payment_amount);

		// the payment should not be overwritten
		assert_noop!(
			Payment::pay(
				RuntimeOrigin::signed(PAYMENT_CREATOR),
				PAYMENT_RECIPENT,
				CURRENCY_ID,
				payment_amount,
				None
			),
			crate::Error::<Test>::PaymentAlreadyInProcess
		);

		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				asset: CURRENCY_ID,
				amount: payment_amount,
				incentive_amount: 2,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 0)),
			})
		);
	});
}
 */
