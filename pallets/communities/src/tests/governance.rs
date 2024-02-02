use super::*;

use frame_support::{parameter_types, traits::OriginTrait};
use pallet_referenda::{BoundedCallOf, Curve, TrackInfoOf};
use parity_scale_codec::Encode;
use sp_runtime::{str_array as s, BoundedVec};

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

	pub ProposalCallRemoveCharlieFromB: BoundedCallOf<Test, ()> = {
		let call: RuntimeCall = Call::<Test>::remove_member { who: CHARLIE, membership_id: MembershipId(COMMUNITY_B, 2) }.into();
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
			.into_iter()
			.map(|n| MembershipId(community_id, n as u32))
			.collect::<Vec<_>>()
			.try_into()
			.expect("Same size in, same size out")
	}

	TestEnvBuilder::new()
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
		.build()
}

mod vote {
	use super::*;

	mod common {
		use super::*;

		pub fn setup() -> sp_io::TestExternalities {
			let mut ext = new_test_ext();

			ext.execute_with(|| {
				assert_ok!(Referenda::submit(
					RuntimeOrigin::signed(ALICE),
					OriginForCommunityA::get(),
					ProposalCallAddBob::get(),
					frame_support::traits::schedule::DispatchTime::After(1),
				));
				assert_ok!(Referenda::submit(
					RuntimeOrigin::signed(BOB),
					OriginForCommunityB::get(),
					ProposalCallRemoveCharlieFromB::get(),
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
						proposal: ProposalCallRemoveCharlieFromB::get(),
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

			ext
		}

		#[test]
		fn fails_if_poll_is_not_ongoing() {
			new_test_ext().execute_with(|| {
				assert_noop!(
					Communities::vote(
						RuntimeOrigin::signed(ALICE),
						MembershipId(COMMUNITY_A, 1),
						0,
						Vote::Standard(true)
					),
					Error::NotOngoing
				);
			});
		}

		#[test]
		fn fails_if_not_a_member() {
			setup().execute_with(|| {
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
		fn fails_if_voting_on_invalid_track() {
			setup().execute_with(|| {
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
		use super::{common::setup as single_membership_setup, *};

		#[test]
		fn poll_passes_on_single_aye() {
			single_membership_setup().execute_with(|| {
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

				tick_block();

				// Proposal is enacted and exeuted
				System::assert_has_event(
					pallet_scheduler::Event::<Test>::Dispatched {
						task: (System::block_number(), 1),
						id: Some([
							90, 97, 76, 213, 143, 172, 44, 37, 211, 244, 120, 171, 6, 186, 7, 72, 240, 129, 154, 150,
							114, 108, 198, 14, 114, 155, 12, 20, 109, 197, 98, 214,
						]),
						result: Ok(()),
					}
					.into(),
				);
			});
		}

		#[test]
		fn poll_rejects_on_single_nay() {
			single_membership_setup().execute_with(|| {
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
				let mut ext = single_membership_setup();

				ext.execute_with(|| {
					// For now, this community will vote membership-based
					assert_ok!(Communities::set_decision_method(
						RuntimeOrigin::root(),
						COMMUNITY_C,
						DecisionMethod::Membership
					));

					tick_block();

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

				tick_block();

				// Proposal is enacted and exeuted
				System::assert_has_event(
					crate::Event::<Test>::MemberRemoved {
						who: CHARLIE,
						membership_id: MembershipId(COMMUNITY_C, 3),
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
}
