use crate::mock::*;
use crate::Escrow as EscrowStore;
use frame_support::{assert_noop, assert_ok};
use orml_traits::MultiCurrency;
use vln_primitives::{EscrowDetail, EscrowState};

#[test]
fn test_create_escrow_works() {
    new_test_ext().execute_with(|| {
        // should fail when escrow is more than balance
        assert_noop!(
            Escrow::create_escrow(
                Origin::signed(ESCROW_CREATOR),
                ESCROW_RECIPENT,
                CURRENCY_ID,
                120
            ),
            orml_tokens::Error::<Test>::BalanceTooLow
        );
        // the escrow amount should not be reserved
        assert_eq!(Tokens::free_balance(CURRENCY_ID, &ESCROW_CREATOR), 100);
        assert_eq!(Tokens::free_balance(CURRENCY_ID, &ESCROW_RECIPENT), 0);

        // should be able to create an escrow with available balance
        assert_ok!(Escrow::create_escrow(
            Origin::signed(ESCROW_CREATOR),
            ESCROW_RECIPENT,
            CURRENCY_ID,
            20
        ));
        assert_eq!(
            EscrowStore::<Test>::get(ESCROW_CREATOR, ESCROW_RECIPENT),
            Some(EscrowDetail {
                asset: CURRENCY_ID,
                amount: 20,
                state: EscrowState::Created
            })
        );
        // the escrow amount should be reserved
        assert_eq!(Tokens::free_balance(CURRENCY_ID, &ESCROW_CREATOR), 80);
        assert_eq!(Tokens::free_balance(CURRENCY_ID, &ESCROW_RECIPENT), 0);

        // the escrow should not be overwritten
        assert_noop!(
            Escrow::create_escrow(
                Origin::signed(ESCROW_CREATOR),
                ESCROW_RECIPENT,
                CURRENCY_ID,
                20
            ),
            crate::Error::<Test>::EscrowAlreadyInProcess
        );

        assert_eq!(
            EscrowStore::<Test>::get(ESCROW_CREATOR, ESCROW_RECIPENT),
            Some(EscrowDetail {
                asset: CURRENCY_ID,
                amount: 20,
                state: EscrowState::Created
            })
        );
    });
}

#[test]
fn test_release_escrow_works() {
    new_test_ext().execute_with(|| {
        // should be able to create an escrow with available balance
        assert_ok!(Escrow::create_escrow(
            Origin::signed(ESCROW_CREATOR),
            ESCROW_RECIPENT,
            CURRENCY_ID,
            40
        ));
        assert_eq!(
            EscrowStore::<Test>::get(ESCROW_CREATOR, ESCROW_RECIPENT),
            Some(EscrowDetail {
                asset: CURRENCY_ID,
                amount: 40,
                state: EscrowState::Created
            })
        );
        // the escrow amount should be reserved
        assert_eq!(Tokens::free_balance(CURRENCY_ID, &ESCROW_CREATOR), 60);
        assert_eq!(Tokens::free_balance(CURRENCY_ID, &ESCROW_RECIPENT), 0);

        // should succeed for valid swap
        assert_ok!(Escrow::release_escrow(
            Origin::signed(ESCROW_CREATOR),
            ESCROW_RECIPENT
        ));
        // the escrow amount should be transferred
        assert_eq!(Tokens::free_balance(CURRENCY_ID, &ESCROW_CREATOR), 60);
        assert_eq!(Tokens::free_balance(CURRENCY_ID, &ESCROW_RECIPENT), 40);
        assert_eq!(Tokens::total_issuance(CURRENCY_ID), 100);

        // should be in released state
        assert_eq!(
            EscrowStore::<Test>::get(ESCROW_CREATOR, ESCROW_RECIPENT),
            Some(EscrowDetail {
                asset: CURRENCY_ID,
                amount: 40,
                state: EscrowState::Released
            })
        );
        // cannot call release again
        assert_noop!(
            Escrow::release_escrow(Origin::signed(ESCROW_CREATOR), ESCROW_RECIPENT),
            crate::Error::<Test>::EscrowAlreadyReleased
        );

        // should be able to create another escrow since previous is released
        assert_ok!(Escrow::create_escrow(
            Origin::signed(ESCROW_CREATOR),
            ESCROW_RECIPENT,
            CURRENCY_ID,
            40
        ));
        // the escrow amount should be reserved
        assert_eq!(Tokens::free_balance(CURRENCY_ID, &ESCROW_CREATOR), 20);
        assert_eq!(Tokens::free_balance(CURRENCY_ID, &ESCROW_RECIPENT), 40);
    });
}
