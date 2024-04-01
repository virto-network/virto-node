use super::*;

use frame_support::{parameter_types, traits::OriginTrait};
use pallet_referenda::{BoundedCallOf, Curve, TrackInfoOf};
use parity_scale_codec::Encode;
use sp_runtime::{str_array as s, BoundedVec, TokenError};

use crate::{
	origin::DecisionMethod,
	types::{Tally, Vote},
	Call,
};
use frame_support::assert_noop;
use pallet_referenda::TrackInfo;
use sp_runtime::Perbill;
use virto_common::MembershipId;

const COMMUNITY_A: CommunityId = CommunityId::new(1);
const COMMUNITY_B: CommunityId = CommunityId::new(2);
const COMMUNITY_C: CommunityId = CommunityId::new(3);
const COMMUNITY_D: CommunityId = CommunityId::new(4);

const COMMUNITY_B_ASSET_ID: AssetId = 2;

const ALICE: AccountId = AccountId::new([1; 32]);
const BOB: AccountId = AccountId::new([2; 32]);
const CHARLIE: AccountId = AccountId::new([3; 32]);

parameter_types! {
	pub OriginForCommunityA: Box<OriginCaller> =
		Box::new(TestEnvBuilder::create_community_origin(&COMMUNITY_A, &DecisionMethod::Membership).caller().clone());
	pub OriginForCommunityB: Box<OriginCaller> =
		Box::new(TestEnvBuilder::create_community_origin(&COMMUNITY_B, &DecisionMethod::CommunityAsset(COMMUNITY_B_ASSET_ID)).caller().clone());
	pub OriginForCommunityC: Box<OriginCaller> =
		Box::new(TestEnvBuilder::create_community_origin(&COMMUNITY_C, &DecisionMethod::NativeToken).caller().clone());
	pub OriginForCommunityD: Box<OriginCaller> =
		Box::new(TestEnvBuilder::create_community_origin(&COMMUNITY_D, &DecisionMethod::Rank).caller().clone());

	pub CommunityTrack: TrackInfoOf<Test> = TrackInfo {
		name: s("Community"),
		max_deciding: 1,
		decision_deposit: 5,
		prepare_period: 1,
		decision_period: 5,
		confirm_period: 1,
		min_enactment_period: 1,
		min_approval: Curve::LinearDecreasing {
			length: Perbill::from_percent(100),
			floor: Perbill::from_percent(50),
			ceil: Perbill::from_percent(100),
		},
		min_support: Curve::LinearDecreasing {
			length: Perbill::from_percent(100),
			floor: Perbill::from_percent(0),
			ceil: Perbill::from_percent(100),
		},
	};

	pub ProposalCallAddBob: BoundedCallOf<Test, ()> = {
		let call: RuntimeCall = Call::<Test>::add_member { who: BOB }.into();
		BoundedCallOf::<Test, ()>::Inline(BoundedVec::truncate_from(call.encode()))
	};

	pub ProposalCallAddAlice: BoundedCallOf<Test, ()> = {
		let call: RuntimeCall = Call::<Test>::add_member { who: ALICE }.into();
		BoundedCallOf::<Test, ()>::Inline(BoundedVec::truncate_from(call.encode()))
	};

	pub ProposalCallRemoveCharlieFromC: BoundedCallOf<Test, ()> = {
		let call: RuntimeCall = Call::<Test>::remove_member { who: CHARLIE, membership_id: MembershipId(COMMUNITY_C, 3) }.into();
		BoundedCallOf::<Test, ()>::Inline(BoundedVec::truncate_from(call.encode()))
	};

	pub ProposalCallPromoteCharlie: BoundedCallOf<Test, ()> = {
		let call: RuntimeCall = Call::<Test>::promote_member { who: CHARLIE, membership_id: MembershipId(COMMUNITY_D, 3) }.into();
		BoundedCallOf::<Test, ()>::Inline(BoundedVec::truncate_from(call.encode()))
	};
}

fn new_test_ext() -> sp_io::TestExternalities {
	fn produce_memberships<const N: usize>(community_id: CommunityId) -> [MembershipId; N] {
		(1..=N)
			.map(|n| MembershipId(community_id, n as u32))
			.collect::<Vec<_>>()
			.try_into()
			.expect("Same size in, same size out")
	}

	let mut t = TestEnvBuilder::new()
		.with_balances(&[(ALICE, 15), (BOB, 15), (CHARLIE, 15)])
		// Membership-based
		.add_community(
			COMMUNITY_A,
			DecisionMethod::Membership,
			&[ALICE],
			&produce_memberships::<3>(COMMUNITY_A),
			Some(CommunityTrack::get()),
		)
		// Community-asset based
		.add_community(
			COMMUNITY_B,
			DecisionMethod::CommunityAsset(COMMUNITY_B_ASSET_ID),
			&[BOB, CHARLIE],
			&produce_memberships::<3>(COMMUNITY_B),
			Some(CommunityTrack::get()),
		)
		.add_asset(
			&COMMUNITY_B_ASSET_ID,
			&Communities::community_account(&COMMUNITY_B),
			true,
			1,
			None,
			Some(vec![(BOB, 10), (CHARLIE, 10)]),
		)
		// Native-asset based
		.add_community(
			COMMUNITY_C,
			DecisionMethod::NativeToken,
			&[ALICE, BOB, CHARLIE],
			&produce_memberships::<3>(COMMUNITY_C),
			Some(CommunityTrack::get()),
		)
		// Rank-based
		.add_community(
			COMMUNITY_D,
			DecisionMethod::Rank,
			&[ALICE, BOB, CHARLIE],
			&produce_memberships::<3>(COMMUNITY_D),
			Some(CommunityTrack::get()),
		)
		.build();

	t.execute_with(|| {
		assert_ok!(Referenda::submit(
			RuntimeOrigin::signed(ALICE),
			OriginForCommunityA::get(),
			ProposalCallAddBob::get(),
			frame_support::traits::schedule::DispatchTime::After(1),
		));
		assert_ok!(Referenda::submit(
			RuntimeOrigin::signed(BOB),
			OriginForCommunityB::get(),
			ProposalCallAddAlice::get(),
			frame_support::traits::schedule::DispatchTime::After(1),
		));
		assert_ok!(Referenda::submit(
			RuntimeOrigin::signed(BOB),
			OriginForCommunityC::get(),
			ProposalCallRemoveCharlieFromC::get(),
			frame_support::traits::schedule::DispatchTime::After(1),
		));

		System::assert_has_event(
			pallet_referenda::Event::<Test>::Submitted {
				index: 0,
				proposal: ProposalCallAddBob::get(),
				track: COMMUNITY_A,
			}
			.into(),
		);
		System::assert_has_event(
			pallet_referenda::Event::<Test>::Submitted {
				index: 1,
				proposal: ProposalCallAddAlice::get(),
				track: COMMUNITY_B,
			}
			.into(),
		);
		System::assert_has_event(
			pallet_referenda::Event::<Test>::Submitted {
				index: 2,
				proposal: ProposalCallRemoveCharlieFromC::get(),
				track: COMMUNITY_C,
			}
			.into(),
		);

		assert_ok!(Referenda::place_decision_deposit(RuntimeOrigin::signed(ALICE), 0));
		assert_ok!(Referenda::place_decision_deposit(RuntimeOrigin::signed(BOB), 1));
		assert_ok!(Referenda::place_decision_deposit(RuntimeOrigin::signed(BOB), 2));

		tick_block();
	});

	t
}

mod vote {
	use super::*;

	mod common {
		use super::*;

		#[test]
		fn fails_if_vote_weight_is_zero() {
			new_test_ext().execute_with(|| {
				assert_noop!(
					Communities::vote(
						RuntimeOrigin::signed(BOB),
						MembershipId(COMMUNITY_B, 1),
						1,
						Vote::AssetBalance(true, COMMUNITY_B_ASSET_ID, 0)
					),
					TokenError::BelowMinimum
				);
			});
		}

		#[test]
		fn fails_if_not_a_member() {
			new_test_ext().execute_with(|| {
				assert_noop!(
					Communities::vote(
						RuntimeOrigin::signed(BOB),
						MembershipId(COMMUNITY_A, 2),
						0,
						Vote::Standard(true)
					),
					Error::NotAMember
				);
			});
		}

		#[test]
		fn fails_if_poll_is_not_ongoing() {
			new_test_ext().execute_with(|| {
				assert_noop!(
					Communities::vote(
						RuntimeOrigin::signed(ALICE),
						MembershipId(COMMUNITY_A, 1),
						256,
						Vote::Standard(true)
					),
					Error::NotOngoing
				);
			});
		}

		#[test]
		fn fails_if_voting_on_invalid_track() {
			new_test_ext().execute_with(|| {
				assert_noop!(
					Communities::vote(
						RuntimeOrigin::signed(BOB),
						MembershipId(COMMUNITY_B, 1),
						0,
						Vote::Standard(true)
					),
					Error::InvalidTrack
				);
			});
		}
	}

	mod membership {
		use fc_traits_memberships::Inspect;

		use super::*;

		#[test]
		fn passing_poll_executes() {
			new_test_ext().execute_with(|| {
				// Before voting, the poll is ongoing
				System::assert_has_event(
					pallet_referenda::Event::<Test>::DecisionStarted {
						index: 0,
						track: COMMUNITY_A,
						proposal: ProposalCallAddBob::get(),
						tally: Tally::default(),
					}
					.into(),
				);

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(ALICE),
					MembershipId(COMMUNITY_A, 1),
					0,
					Vote::Standard(true)
				));

				tick_block();

				// After voting, the poll starts confirmation
				System::assert_has_event(pallet_referenda::Event::<Test>::ConfirmStarted { index: 0 }.into());

				tick_block();

				// After confirmation, vote should be completed and approved
				System::assert_has_event(
					pallet_referenda::Event::<Test>::Confirmed {
						index: 0,
						tally: Tally {
							ayes: 1,
							nays: 0,
							bare_ayes: 1,
							..Default::default()
						},
					}
					.into(),
				);

				let community_account = Communities::community_account(&COMMUNITY_A);
				let (_, membership_id) = Nfts::user_memberships(&community_account, None)
					.next()
					.expect("CommunityA should still have memberships");

				tick_block();

				// Proposal is enacted and exeuted
				System::assert_has_event(
					crate::Event::<Test>::MemberAdded {
						who: BOB,
						membership_id,
					}
					.into(),
				);
			});
		}

		#[test]
		fn poll_rejects_on_single_nay() {
			new_test_ext().execute_with(|| {
				// Before voting, the poll is ongoing
				System::assert_has_event(
					pallet_referenda::Event::<Test>::DecisionStarted {
						index: 0,
						track: COMMUNITY_A,
						proposal: ProposalCallAddBob::get(),
						tally: Tally::default(),
					}
					.into(),
				);

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(ALICE),
					MembershipId(COMMUNITY_A, 1),
					0,
					Vote::Standard(false)
				));

				tick_blocks(5);

				// After voting, the poll should be completed and approved
				System::assert_has_event(
					pallet_referenda::Event::<Test>::Rejected {
						index: 0,
						tally: Tally {
							ayes: 0,
							nays: 1,
							bare_ayes: 0,
							..Default::default()
						},
					}
					.into(),
				);
			});
		}

		#[test]
		fn tie_breaking_works() {
			fn run_referenda() -> sp_io::TestExternalities {
				let mut ext = new_test_ext();

				ext.execute_with(|| {
					// For now, this community will vote membership-based
					assert_ok!(Communities::set_decision_method(
						RuntimeOrigin::root(),
						COMMUNITY_C,
						DecisionMethod::Membership
					));

					// Before voting, the poll is ongoing
					System::assert_has_event(
						pallet_referenda::Event::<Test>::DecisionStarted {
							index: 2,
							track: COMMUNITY_C,
							proposal: ProposalCallRemoveCharlieFromC::get(),
							tally: Tally::default(),
						}
						.into(),
					);

					assert_ok!(Communities::vote(
						RuntimeOrigin::signed(ALICE),
						MembershipId(COMMUNITY_C, 1),
						2,
						Vote::Standard(true)
					));

					tick_block();

					assert_ok!(Communities::vote(
						RuntimeOrigin::signed(CHARLIE),
						MembershipId(COMMUNITY_C, 3),
						2,
						Vote::Standard(false)
					));

					tick_blocks(4);

					// After voting, the poll starts confirmation
					System::assert_has_event(pallet_referenda::Event::<Test>::ConfirmStarted { index: 2 }.into());
				});

				ext
			}

			run_referenda().execute_with(|| {
				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(BOB),
					MembershipId(COMMUNITY_C, 2),
					2,
					Vote::Standard(true)
				));

				tick_block();

				// After confirmation, vote should be completed and approved
				System::assert_has_event(
					pallet_referenda::Event::<Test>::Confirmed {
						index: 2,
						tally: Tally {
							ayes: 2,
							nays: 1,
							bare_ayes: 2,
							..Default::default()
						},
					}
					.into(),
				);
			});

			run_referenda().execute_with(|| {
				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(BOB),
					MembershipId(COMMUNITY_C, 2),
					2,
					Vote::Standard(false)
				));

				tick_block();

				// After voting, the poll starts confirmation
				System::assert_has_event(
					pallet_referenda::Event::<Test>::Rejected {
						index: 2,
						tally: Tally {
							ayes: 1,
							nays: 2,
							bare_ayes: 1,
							..Default::default()
						},
					}
					.into(),
				);
			});
		}
	}

	mod asset_balance {
		use super::*;
		use frame_support::traits::fungibles::MutateHold;

		#[test]
		fn fails_if_not_enough_balance() {
			new_test_ext().execute_with(|| {
				// Cannot keep free funds lower than min. balance
				assert_noop!(
					Communities::vote(
						RuntimeOrigin::signed(CHARLIE),
						MembershipId(COMMUNITY_B, 2),
						1,
						Vote::AssetBalance(false, COMMUNITY_B_ASSET_ID, 10)
					),
					TokenError::FundsUnavailable
				);
			});
		}

		#[test]
		fn holds_cannot_overlap() {
			new_test_ext().execute_with(|| {
				// If already holds should be overlapable
				assert_ok!(Assets::hold(
					COMMUNITY_B_ASSET_ID,
					&pallet_preimage::HoldReason::Preimage.into(),
					&CHARLIE,
					6,
				));

				// Before voting, the poll is ongoing
				assert_noop!(
					Communities::vote(
						RuntimeOrigin::signed(CHARLIE),
						MembershipId(COMMUNITY_B, 2),
						1,
						Vote::AssetBalance(false, COMMUNITY_B_ASSET_ID, 5)
					),
					TokenError::FundsUnavailable
				);
			});
		}

		#[test]
		fn passes_with_approval_and_support() {
			new_test_ext().execute_with(|| {
				// Before voting, the poll is ongoing
				System::assert_has_event(
					pallet_referenda::Event::<Test>::DecisionStarted {
						index: 1,
						track: COMMUNITY_B,
						proposal: ProposalCallAddAlice::get(),
						tally: Tally::default(),
					}
					.into(),
				);

				// We're going to vote high enough to pass to confirmation immediately:
				// 66% approval / 10% of support

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(BOB),
					MembershipId(COMMUNITY_B, 1),
					1,
					Vote::AssetBalance(true, COMMUNITY_B_ASSET_ID, 6)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(CHARLIE),
					MembershipId(COMMUNITY_B, 2),
					1,
					Vote::AssetBalance(true, COMMUNITY_B_ASSET_ID, 6)
				));

				tick_block();

				System::assert_has_event(pallet_referenda::Event::<Test>::ConfirmStarted { index: 1 }.into());

				tick_block();

				System::assert_has_event(
					pallet_referenda::Event::<Test>::Confirmed {
						index: 1,
						tally: Tally {
							ayes: 12,
							nays: 0,
							bare_ayes: 12,
							..Default::default()
						},
					}
					.into(),
				);
			});
		}

		#[test]
		fn passes_with_approval_but_not_support() {
			new_test_ext().execute_with(|| {
				// Before voting, the poll is ongoing
				System::assert_has_event(
					pallet_referenda::Event::<Test>::DecisionStarted {
						index: 1,
						track: COMMUNITY_B,
						proposal: ProposalCallAddAlice::get(),
						tally: Tally::default(),
					}
					.into(),
				);

				// We're going to vote high enough to have a pass in approval, but not enough to
				// pass in support until decision period ends: 66% approval / 10% of support

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(BOB),
					MembershipId(COMMUNITY_B, 1),
					1,
					Vote::AssetBalance(true, COMMUNITY_B_ASSET_ID, 2)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(CHARLIE),
					MembershipId(COMMUNITY_B, 2),
					1,
					Vote::AssetBalance(false, COMMUNITY_B_ASSET_ID, 1)
				));

				tick_blocks(4);

				System::assert_has_event(pallet_referenda::Event::<Test>::ConfirmStarted { index: 1 }.into());

				tick_block();

				System::assert_has_event(
					pallet_referenda::Event::<Test>::Confirmed {
						index: 1,
						tally: Tally {
							ayes: 2,
							nays: 1,
							bare_ayes: 2,
							..Default::default()
						},
					}
					.into(),
				);
			});
		}

		#[test]
		fn voter_can_change_decision_over_time() {
			new_test_ext().execute_with(|| {
				// Before voting, the poll is ongoing
				System::assert_has_event(
					pallet_referenda::Event::<Test>::DecisionStarted {
						index: 1,
						track: COMMUNITY_B,
						proposal: ProposalCallAddAlice::get(),
						tally: Tally::default(),
					}
					.into(),
				);

				tick_block();

				// We're going to vote high in a series of three votes, each one attempting to
				// turn the poll over.

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(CHARLIE),
					MembershipId(COMMUNITY_B, 2),
					1,
					Vote::AssetBalance(true, COMMUNITY_B_ASSET_ID, 6)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(BOB),
					MembershipId(COMMUNITY_B, 1),
					1,
					Vote::AssetBalance(false, COMMUNITY_B_ASSET_ID, 7)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(CHARLIE),
					MembershipId(COMMUNITY_B, 2),
					1,
					Vote::AssetBalance(true, COMMUNITY_B_ASSET_ID, 8)
				));

				tick_blocks(3);

				System::assert_has_event(
					pallet_referenda::Event::<Test>::Confirmed {
						index: 1,
						tally: Tally {
							ayes: 8,
							nays: 7,
							bare_ayes: 8,
							..Default::default()
						},
					}
					.into(),
				);
			});
		}
	}

	mod native_balance {
		use super::*;
		use frame_support::traits::fungible::MutateFreeze;

		#[test]
		fn fails_if_not_enough_balance() {
			new_test_ext().execute_with(|| {
				assert_noop!(
					Communities::vote(
						RuntimeOrigin::signed(BOB),
						MembershipId(COMMUNITY_C, 2),
						2,
						Vote::NativeBalance(true, 16)
					),
					TokenError::FundsUnavailable
				);
			});
		}

		#[test]
		fn locks_can_overlap() {
			new_test_ext().execute_with(|| {
				// Suppose CHARLIE has already casted a vote on other poll (let's call it 4)
				assert_ok!(Balances::set_freeze(
					&pallet_preimage::HoldReason::Preimage.into(),
					&CHARLIE,
					12
				));

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(CHARLIE),
					MembershipId(COMMUNITY_C, 3),
					2,
					Vote::NativeBalance(true, 11)
				));
			});
		}

		#[test]
		fn rejects_on_most_nays() {
			new_test_ext().execute_with(|| {
				// Before voting, the poll is ongoing
				System::assert_has_event(
					pallet_referenda::Event::<Test>::DecisionStarted {
						index: 2,
						track: COMMUNITY_C,
						proposal: ProposalCallRemoveCharlieFromC::get(),
						tally: Tally::default(),
					}
					.into(),
				);

				tick_block();

				// BOB won't be able to vote, since they don't have enough funds to do so
				// ALICE has a limited support they can put, so CHARLIE will be able to
				// cast a majority nay vote.

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(CHARLIE),
					MembershipId(COMMUNITY_C, 3),
					2,
					Vote::NativeBalance(false, 14)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(ALICE),
					MembershipId(COMMUNITY_C, 1),
					2,
					Vote::NativeBalance(true, 7)
				));

				tick_blocks(3);

				System::assert_has_event(
					pallet_referenda::Event::<Test>::Rejected {
						index: 2,
						tally: Tally {
							ayes: 7,
							nays: 14,
							bare_ayes: 7,
							..Default::default()
						},
					}
					.into(),
				);
			});
		}

		#[test]
		fn voter_can_change_decision() {
			new_test_ext().execute_with(|| {
				// Before voting, the poll is ongoing
				System::assert_has_event(
					pallet_referenda::Event::<Test>::DecisionStarted {
						index: 2,
						track: COMMUNITY_C,
						proposal: ProposalCallRemoveCharlieFromC::get(),
						tally: Tally::default(),
					}
					.into(),
				);

				tick_block();

				// We're going to vote high in a series of three votes, each one attempting to
				// turn the poll over.

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(CHARLIE),
					MembershipId(COMMUNITY_C, 3),
					2,
					Vote::NativeBalance(false, 6)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(ALICE),
					MembershipId(COMMUNITY_C, 1),
					2,
					Vote::NativeBalance(true, 7)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(CHARLIE),
					MembershipId(COMMUNITY_C, 3),
					2,
					Vote::NativeBalance(false, 8)
				));

				tick_blocks(2);

				System::assert_has_event(
					pallet_referenda::Event::<Test>::Rejected {
						index: 2,
						tally: Tally {
							ayes: 7,
							nays: 8,
							bare_ayes: 7,
							..Default::default()
						},
					}
					.into(),
				);
			});
		}
	}

	mod rank {
		use frame_support::traits::Polling;

		use super::*;

		fn new_test_ext() -> sp_io::TestExternalities {
			let mut ext = super::new_test_ext();

			ext.execute_with(|| {
				assert_ok!(Communities::promote_member(
					Into::<RuntimeOrigin>::into(*OriginForCommunityD::get()),
					ALICE,
					MembershipId(COMMUNITY_D, 1)
				));
				assert_ok!(Communities::promote_member(
					Into::<RuntimeOrigin>::into(*OriginForCommunityD::get()),
					BOB,
					MembershipId(COMMUNITY_D, 2)
				));
				assert_ok!(Communities::promote_member(
					Into::<RuntimeOrigin>::into(*OriginForCommunityD::get()),
					CHARLIE,
					MembershipId(COMMUNITY_D, 3)
				));

				assert_ok!(Referenda::submit(
					RuntimeOrigin::signed(CHARLIE),
					OriginForCommunityD::get(),
					ProposalCallPromoteCharlie::get(),
					frame_support::traits::schedule::DispatchTime::After(1),
				));

				System::assert_has_event(
					pallet_referenda::Event::<Test>::Submitted {
						index: 3,
						proposal: ProposalCallPromoteCharlie::get(),
						track: COMMUNITY_D,
					}
					.into(),
				);

				assert_ok!(Referenda::place_decision_deposit(RuntimeOrigin::signed(CHARLIE), 3));

				tick_block();
			});

			ext
		}

		#[test]
		fn it_works() {
			new_test_ext().execute_with(|| {
				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(ALICE),
					MembershipId(COMMUNITY_D, 1),
					3,
					Vote::Standard(true)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(BOB),
					MembershipId(COMMUNITY_D, 2),
					3,
					Vote::Standard(false)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(CHARLIE),
					MembershipId(COMMUNITY_D, 3),
					3,
					Vote::Standard(true)
				));

				tick_blocks(2);

				System::assert_has_event(pallet_referenda::Event::<Test>::ConfirmStarted { index: 3 }.into());
			});
		}

		#[test]
		fn it_works_with_different_ranks() {
			new_test_ext().execute_with(|| {
				assert_ok!(Communities::promote_member(
					Into::<RuntimeOrigin>::into(*OriginForCommunityD::get()),
					ALICE,
					MembershipId(COMMUNITY_D, 1)
				));

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(ALICE),
					MembershipId(COMMUNITY_D, 1),
					3,
					Vote::Standard(false)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(BOB),
					MembershipId(COMMUNITY_D, 2),
					3,
					Vote::Standard(true)
				));

				tick_block();

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(CHARLIE),
					MembershipId(COMMUNITY_D, 3),
					3,
					Vote::Standard(true)
				));

				assert_eq!(
					Referenda::as_ongoing(3).expect("the poll was initiated; qed").0,
					Tally {
						ayes: 2,
						nays: 2,
						bare_ayes: 2,
						..Default::default()
					}
				)
			});
		}
	}
}

mod remove_vote {
	use frame_support::traits::{fungible::Inspect, Polling};

	use super::*;

	#[test]
	fn fails_if_not_a_member() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				Communities::remove_vote(RuntimeOrigin::signed(BOB), MembershipId(COMMUNITY_A, 2), 0,),
				Error::NotAMember
			);
		});
	}

	#[test]
	fn fails_if_trying_to_remove_vote_from_invalid_track() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				Communities::remove_vote(RuntimeOrigin::signed(ALICE), MembershipId(COMMUNITY_A, 1), 1),
				Error::InvalidTrack
			);
		});
	}

	#[test]
	fn fails_if_poll_is_no_vote_casted() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				Communities::remove_vote(RuntimeOrigin::signed(ALICE), MembershipId(COMMUNITY_A, 1), 0),
				Error::NoVoteCasted
			);
		});
	}

	#[test]
	fn it_works() {
		new_test_ext().execute_with(|| {
			assert_ok!(Communities::vote(
				RuntimeOrigin::signed(ALICE),
				MembershipId(COMMUNITY_A, 1),
				0,
				Vote::Standard(true)
			));

			tick_block();

			assert_ok!(Communities::remove_vote(
				RuntimeOrigin::signed(ALICE),
				MembershipId(COMMUNITY_A, 1),
				0
			));

			System::assert_has_event(
				crate::Event::<Test>::VoteRemoved {
					who: ALICE,
					poll_index: 0,
				}
				.into(),
			);

			assert_eq!(
				Referenda::as_ongoing(0).expect("we already created poll 0; qed").0,
				Tally::default()
			);
		});

		new_test_ext().execute_with(|| {
			assert_ok!(Communities::vote(
				RuntimeOrigin::signed(ALICE),
				MembershipId(COMMUNITY_C, 1),
				2,
				Vote::NativeBalance(true, 15)
			));

			assert_eq!(
				Balances::reducible_balance(
					&ALICE,
					frame_support::traits::tokens::Preservation::Expendable,
					frame_support::traits::tokens::Fortitude::Polite
				),
				0
			);

			tick_block();

			assert_ok!(Communities::remove_vote(
				RuntimeOrigin::signed(ALICE),
				MembershipId(COMMUNITY_C, 1),
				2
			));

			System::assert_has_event(
				crate::Event::<Test>::VoteRemoved {
					who: ALICE,
					poll_index: 2,
				}
				.into(),
			);

			assert_eq!(
				Referenda::as_ongoing(2).expect("we already created poll 2; qed").0,
				Tally::default()
			);

			assert_eq!(
				Balances::reducible_balance(
					&ALICE,
					frame_support::traits::tokens::Preservation::Expendable,
					frame_support::traits::tokens::Fortitude::Polite
				),
				7
			);
		});
	}
}

mod unlock {
	use super::*;

	#[test]
	fn fails_if_trying_to_unlock_on_an_ongoing_poll() {
		new_test_ext().execute_with(|| {
			// Since BOB never casted a vote, a lock wasn't put in place
			assert_noop!(
				Communities::unlock(RuntimeOrigin::signed(BOB), 1),
				Error::AlreadyOngoing
			);
		});
	}

	#[test]
	fn fails_if_no_locks_in_place() {
		new_test_ext().execute_with(|| {
			tick_blocks(6);

			// Since BOB never casted a vote, a lock wasn't put in place
			assert_noop!(
				Communities::unlock(RuntimeOrigin::signed(BOB), 1),
				Error::NoLocksInPlace
			);

			// Since CHARLIE never casted a vote, a freeze wasn't put in place
			assert_noop!(
				Communities::unlock(RuntimeOrigin::signed(CHARLIE), 2),
				Error::NoLocksInPlace
			);
		});
	}

	#[test]
	fn it_works() {
		new_test_ext().execute_with(|| {
			assert_ok!(Communities::vote(
				RuntimeOrigin::signed(BOB),
				MembershipId(COMMUNITY_B, 1),
				1,
				Vote::AssetBalance(true, COMMUNITY_B_ASSET_ID, 9)
			));

			assert_ok!(Communities::vote(
				RuntimeOrigin::signed(CHARLIE),
				MembershipId(COMMUNITY_C, 3),
				2,
				Vote::NativeBalance(true, 15)
			));

			tick_blocks(6);

			assert_ok!(Communities::unlock(RuntimeOrigin::signed(BOB), 1));

			assert_ok!(Communities::unlock(RuntimeOrigin::signed(CHARLIE), 2));
		});
	}
}
