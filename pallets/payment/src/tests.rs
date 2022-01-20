use crate::{
	mock::*,
	types::{PaymentDetail, PaymentState},
	Payment as PaymentStore,
};
use frame_support::{assert_noop, assert_ok};
use orml_traits::MultiCurrency;

fn last_event() -> Event {
	System::events().pop().expect("Event expected").event
}

#[test]
fn test_create_payment_works() {
	new_test_ext().execute_with(|| {
		// the payment amount should not be reserved
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_CREATOR), 100);
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_RECIPENT), 0);

		// should be able to create a payment with available balance
		assert_ok!(Payment::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			CURRENCY_ID,
			20,
		));
		// assert_eq!(
		// 	last_event(),
		// 	crate::Event::<Test>::PaymentCreated {
		// 		from: PAYMENT_CREATOR,
		// 		asset: CURRENCY_ID,
		// 		amount: 20
		// 	}
		// 	.into()
		// );
		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				asset: CURRENCY_ID,
				amount: 20,
				incentive_amount: 2,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 0)),
				remark: None
			})
		);
		// the payment amount should be reserved correctly
		// the amount + incentive should be removed from the sender account
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_CREATOR), 78);
		// the incentive amount should be reserved in the sender account
		assert_eq!(Tokens::total_balance(CURRENCY_ID, &PAYMENT_CREATOR), 80);
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_RECIPENT), 0);
		// the transferred amount should be reserved in the recipent account
		assert_eq!(Tokens::total_balance(CURRENCY_ID, &PAYMENT_RECIPENT), 20);

		// the payment should not be overwritten
		assert_noop!(
			Payment::pay(Origin::signed(PAYMENT_CREATOR), PAYMENT_RECIPENT, CURRENCY_ID, 20,),
			crate::Error::<Test>::PaymentAlreadyInProcess
		);

		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				asset: CURRENCY_ID,
				amount: 20,
				incentive_amount: 2,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 0)),
				remark: None
			})
		);
	});
}

#[test]
fn test_cancel_payment_works() {
	new_test_ext().execute_with(|| {
		// should be able to create a payment with available balance
		assert_ok!(Payment::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			CURRENCY_ID,
			40,
		));
		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				asset: CURRENCY_ID,
				amount: 40,
				incentive_amount: 4,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 0)),
				remark: None
			})
		);
		// the payment amount should be reserved
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_CREATOR), 56);
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_RECIPENT), 0);

		// cancel should fail when called by user
		assert_noop!(
			Payment::cancel(Origin::signed(PAYMENT_CREATOR), PAYMENT_RECIPENT),
			crate::Error::<Test>::InvalidPayment
		);

		// cancel should succeed when caller is the recipent
		assert_ok!(Payment::cancel(Origin::signed(PAYMENT_RECIPENT), PAYMENT_CREATOR));
		// assert_eq!(
		// 	last_event(),
		// 	crate::Event::<Test>::PaymentCancelled { from: PAYMENT_CREATOR, to: PAYMENT_RECIPENT }
		// 		.into()
		// );
		// the payment amount should be released back to creator
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_CREATOR), 100);
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_RECIPENT), 0);
		assert_eq!(Tokens::total_issuance(CURRENCY_ID), 100);

		// should be released from storage
		assert_eq!(PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT), None);
	});
}

#[test]
fn test_release_payment_works() {
	new_test_ext().execute_with(|| {
		// should be able to create a payment with available balance
		assert_ok!(Payment::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			CURRENCY_ID,
			40,
		));
		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				asset: CURRENCY_ID,
				amount: 40,
				incentive_amount: 4,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 0)),
				remark: None
			})
		);
		// the payment amount should be reserved
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_CREATOR), 56);
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_RECIPENT), 0);

		// should succeed for valid payment
		assert_ok!(Payment::release(Origin::signed(PAYMENT_CREATOR), PAYMENT_RECIPENT));
		// the payment amount should be transferred
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_CREATOR), 60);
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_RECIPENT), 40);
		assert_eq!(Tokens::total_issuance(CURRENCY_ID), 100);

		// should be deleted from storage
		assert_eq!(PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT), None);

		// should be able to create another payment since previous is released
		assert_ok!(Payment::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			CURRENCY_ID,
			40,
		));
		// the payment amount should be reserved
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_CREATOR), 16);
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_RECIPENT), 40);
	});
}

#[test]
fn test_set_state_payment_works() {
	new_test_ext().execute_with(|| {
		// should be able to create a payment with available balance
		assert_ok!(Payment::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			CURRENCY_ID,
			40,
		));

		// should fail for non whitelisted caller
		assert_noop!(
			Payment::resolve_cancel_payment(
				Origin::signed(PAYMENT_CREATOR),
				PAYMENT_CREATOR,
				PAYMENT_RECIPENT,
			),
			crate::Error::<Test>::InvalidAction
		);

		// should be able to release a payment
		assert_ok!(Payment::resolve_release_payment(
			Origin::signed(RESOLVER_ACCOUNT),
			PAYMENT_CREATOR,
			PAYMENT_RECIPENT,
		));

		// the payment amount should be transferred
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_CREATOR), 60);
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_RECIPENT), 40);
		assert_eq!(Tokens::total_issuance(CURRENCY_ID), 100);

		// should be removed from storage
		assert_eq!(PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT), None);

		assert_ok!(Payment::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			CURRENCY_ID,
			40,
		));

		// should be able to cancel a payment
		assert_ok!(Payment::resolve_cancel_payment(
			Origin::signed(RESOLVER_ACCOUNT),
			PAYMENT_CREATOR,
			PAYMENT_RECIPENT,
		));

		// the payment amount should be transferred
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_CREATOR), 60);
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_RECIPENT), 40);
		assert_eq!(Tokens::total_issuance(CURRENCY_ID), 100);

		// should be released from storage
		assert_eq!(PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT), None);
	});
}

#[test]
fn test_charging_fee_payment_works() {
	new_test_ext().execute_with(|| {
		// should be able to create a payment with available balance
		assert_ok!(Payment::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT_FEE_CHARGED,
			CURRENCY_ID,
			40,
		));
		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT_FEE_CHARGED),
			Some(PaymentDetail {
				asset: CURRENCY_ID,
				amount: 40,
				incentive_amount: 4,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 4)),
				remark: None
			})
		);
		// the payment amount should be reserved
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_CREATOR), 52);
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_RECIPENT_FEE_CHARGED), 0);

		// should succeed for valid payment
		assert_ok!(Payment::release(Origin::signed(PAYMENT_CREATOR), PAYMENT_RECIPENT_FEE_CHARGED));
		// the payment amount should be transferred
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_CREATOR), 56);
		assert_eq!(Tokens::total_balance(CURRENCY_ID, &PAYMENT_CREATOR), 56);
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_RECIPENT_FEE_CHARGED), 40);
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &FEE_RECIPIENT_ACCOUNT), 4);
		assert_eq!(Tokens::total_issuance(CURRENCY_ID), 100);
	});
}

#[test]
fn test_charging_fee_payment_works_when_canceled() {
	new_test_ext().execute_with(|| {
		// should be able to create a payment with available balance
		assert_ok!(Payment::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT_FEE_CHARGED,
			CURRENCY_ID,
			40,
		));
		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT_FEE_CHARGED),
			Some(PaymentDetail {
				asset: CURRENCY_ID,
				amount: 40,
				incentive_amount: 4,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 4)),
				remark: None
			})
		);
		// the payment amount should be reserved
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_CREATOR), 52);
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_RECIPENT_FEE_CHARGED), 0);

		// should succeed for valid payment
		assert_ok!(Payment::cancel(Origin::signed(PAYMENT_RECIPENT_FEE_CHARGED), PAYMENT_CREATOR));
		// the payment amount should be transferred
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_CREATOR), 100);
		assert_eq!(Tokens::total_balance(CURRENCY_ID, &PAYMENT_CREATOR), 100);
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_RECIPENT_FEE_CHARGED), 0);
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &FEE_RECIPIENT_ACCOUNT), 0);
		assert_eq!(Tokens::total_issuance(CURRENCY_ID), 100);
	});
}

#[test]
fn test_remark_too_large_should_be_rejected() {
	new_test_ext().execute_with(|| {
		// payments with larger than limit remarks should be rejected
		assert_noop!(
			Payment::pay_with_remark(
				Origin::signed(PAYMENT_CREATOR),
				PAYMENT_RECIPENT_FEE_CHARGED,
				CURRENCY_ID,
				40,
				vec![1; 51]
			),
			crate::Error::<Test>::RemarkTooLarge
		);
	});
}

#[test]
fn test_pay_with_remark_works() {
	new_test_ext().execute_with(|| {
		// should be able to create a payment with available balance
		assert_ok!(Payment::pay_with_remark(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			CURRENCY_ID,
			20,
			"test".into()
		));
		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				asset: CURRENCY_ID,
				amount: 20,
				incentive_amount: 2,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 0)),
				remark: Some("test".into())
			})
		);
		// the payment amount should be reserved correctly
		// the amount + incentive should be removed from the sender account
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_CREATOR), 78);
		// the incentive amount should be reserved in the sender account
		assert_eq!(Tokens::total_balance(CURRENCY_ID, &PAYMENT_CREATOR), 80);
		assert_eq!(Tokens::free_balance(CURRENCY_ID, &PAYMENT_RECIPENT), 0);
		// the transferred amount should be reserved in the recipent account
		assert_eq!(Tokens::total_balance(CURRENCY_ID, &PAYMENT_RECIPENT), 20);

		// the payment should not be overwritten
		assert_noop!(
			Payment::pay(Origin::signed(PAYMENT_CREATOR), PAYMENT_RECIPENT, CURRENCY_ID, 20,),
			crate::Error::<Test>::PaymentAlreadyInProcess
		);
	});
}

#[test]
fn test_do_not_overwrite_logic_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Payment::pay_with_remark(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			CURRENCY_ID,
			20,
			"test".into()
		));

		assert_noop!(
			Payment::pay(Origin::signed(PAYMENT_CREATOR), PAYMENT_RECIPENT, CURRENCY_ID, 20,),
			crate::Error::<Test>::PaymentAlreadyInProcess
		);

		// set payment state to NeedsReview
		PaymentStore::<Test>::insert(
			PAYMENT_CREATOR,
			PAYMENT_RECIPENT,
			PaymentDetail {
				asset: CURRENCY_ID,
				amount: 20,
				incentive_amount: 2,
				state: PaymentState::NeedsReview,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 0)),
				remark: Some("test".into()),
			},
		);

		// the payment should not be overwritten
		assert_noop!(
			Payment::pay(Origin::signed(PAYMENT_CREATOR), PAYMENT_RECIPENT, CURRENCY_ID, 20,),
			crate::Error::<Test>::PaymentNeedsReview
		);
	});
}
