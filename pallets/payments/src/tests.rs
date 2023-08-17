use crate::{
	mock::*,
	types::{PaymentDetail, PaymentState},
	weights::WeightInfo,
	Payment as PaymentStore, PaymentHandler, ScheduledTask, Task,
};
use frame_support::{assert_ok, traits::fungibles};
use frame_system::RawOrigin;
use sp_runtime::BoundedVec;

#[test]
fn test_pay_and_release_works() {
	new_test_ext().execute_with(|| {
		let asset = 0;
		let admin = 1;
		let source = 2; // account with own deposit
		let dest = 21; // account with own deposit
		let creator_initial_balance = 100;
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), asset, admin, true, 1));
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(admin),
			asset,
			source,
			creator_initial_balance
		));

		let payment_amount = 20;
		let expected_incentive_amount: u64 = payment_amount / INCENTIVE_PERCENTAGE as u64;
		println!("expected_incentive_amount: {:?}", expected_incentive_amount);

		let remark: BoundedVec<u8, MaxRemarkLength> = BoundedVec::truncate_from(b"remark".to_vec());

		Payments::pay(
			RuntimeOrigin::signed(source),
			dest,
			asset,
			payment_amount,
			Some(remark),
			HoldIdentifiers::TransferPayment,
		)
		.unwrap();

		let fee = 4u64;

		let payment = PaymentStore::<Test>::get(source, dest);
		println!("payment: {:?}", payment);

		assert_eq!(
			<Assets as fungibles::InspectHold<_>>::balance_on_hold(asset, &HoldIdentifiers::TransferPayment, &dest),
			20
		);
		assert_eq!(
			<Assets as fungibles::InspectHold<_>>::balance_on_hold(asset, &HoldIdentifiers::TransferPayment, &source),
			2
		);

		Payments::release(RuntimeOrigin::signed(source), dest, HoldIdentifiers::TransferPayment).unwrap();

		assert_eq!(<Assets as fungibles::Inspect<_>>::balance(asset, &dest), 16);
		let expected_source = creator_initial_balance - payment_amount - fee + expected_incentive_amount;
		assert_eq!(
			<Assets as fungibles::Inspect<_>>::balance(asset, &source),
			expected_source
		);
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
