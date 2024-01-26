use frame_support::parameter_types;
use pallet_referenda::{BoundedCallOf, Curve, TrackInfoOf};
use parity_scale_codec::Encode;
use sp_runtime::{str_array as s, BoundedVec};

use super::*;
use crate::{types::Vote, Call};
use frame_support::assert_noop;
use pallet_referenda::TrackInfo;
use sp_runtime::Perbill;
use virto_common::MembershipId;

const COMMUNITY_MEMBER_1: AccountId = AccountId::new([1; 32]);
const COMMUNITY_MEMBER_2: AccountId = AccountId::new([2; 32]);
const MEMBERSHIP_1: MembershipId = MembershipId(COMMUNITY, 1);
const MEMBERSHIP_2: MembershipId = MembershipId(COMMUNITY, 2);

const COMMUNITY_ASSET_ID: AssetId = 1;

parameter_types! {
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

	pub Proposal: BoundedCallOf<Test, ()> = {
		let call: RuntimeCall = Call::<Test>::add_member { who: COMMUNITY_MEMBER_2 }.into();
		BoundedCallOf::<Test, ()>::Inline(BoundedVec::truncate_from(call.encode()))
	};
}

fn new_test_ext() -> sp_io::TestExternalities {
	TestEnvBuilder::new()
		.with_balances(&[(COMMUNITY_MEMBER_1, 10), (COMMUNITY_MEMBER_2, 10)])
		.add_asset(
			&COMMUNITY_ASSET_ID,
			&Communities::community_account(&COMMUNITY),
			true,
			1,
			None,
			Some(vec![(COMMUNITY_MEMBER_1, 10)]),
		)
		.with_memberships(&[MEMBERSHIP_1])
		.with_members(&[COMMUNITY_MEMBER_1])
		.build()
}

mod vote {
	use super::*;

	mod common {
		use super::*;

		pub fn setup() -> sp_io::TestExternalities {
			let mut ext = new_test_ext();

			ext.execute_with(|| {
				assert_ok!(Tracks::insert(
					RuntimeOrigin::root(),
					COMMUNITY,
					CommunityTrack::get(),
					COMMUNITY_ORGIN
				));

				assert_ok!(Referenda::submit(
					RuntimeOrigin::signed(COMMUNITY_MEMBER_1),
					Box::new(COMMUNITY_ORGIN),
					Proposal::get(),
					frame_support::traits::schedule::DispatchTime::After(1),
				));

				System::assert_has_event(
					pallet_referenda::Event::<Test>::Submitted {
						index: 0,
						proposal: Proposal::get(),
						track: COMMUNITY,
					}
					.into(),
				);

				assert_ok!(Referenda::place_decision_deposit(
					RuntimeOrigin::signed(COMMUNITY_MEMBER_1),
					0
				));

				next_block();
			});

			ext
		}

		#[test]
		fn fails_if_not_a_member() {
			setup().execute_with(|| {
				assert_noop!(
					Communities::vote(
						RuntimeOrigin::signed(COMMUNITY_MEMBER_2),
						MEMBERSHIP_2,
						0,
						Vote::Standard(true)
					),
					Error::NotAMember
				);
			});
		}

		#[test]
		fn fails_if_poll_is_not_ongoing() {
			setup().execute_with(|| {
				assert_noop!(
					Communities::vote(
						RuntimeOrigin::signed(COMMUNITY_MEMBER_1),
						MEMBERSHIP_1,
						1,
						Vote::Standard(true)
					),
					Error::NotOngoing
				);
			});
		}
	}

	mod membership {
		use crate::types::Tally;

		use super::{common::setup as single_membership_setup, *};

		#[test]
		fn poll_passes_on_single_aye() {
			single_membership_setup().execute_with(|| {
				// Before voting, the poll is ongoing
				System::assert_has_event(
					pallet_referenda::Event::<Test>::DecisionStarted {
						index: 0,
						track: COMMUNITY,
						proposal: Proposal::get(),
						tally: Tally::default(),
					}
					.into(),
				);

				assert_ok!(Communities::vote(
					RuntimeOrigin::signed(COMMUNITY_MEMBER_1),
					MEMBERSHIP_1,
					0,
					Vote::Standard(true)
				));

				next_block();

				// After voting, the poll should be completed and approved
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

				next_block();

				// Proposal is enacted and exeuted
				System::assert_has_event(
					pallet_scheduler::Event::<Test>::Dispatched {
						task: (System::block_number(), 1),
						id: Some([
							90, 97, 76, 213, 143, 172, 44, 37, 211, 244, 120, 171, 6, 186, 7, 72, 240, 129, 154, 150,
							114, 108, 198, 14, 114, 155, 12, 20, 109, 197, 98, 214,
						]),
						result: Err(Error::CommunityAtCapacity.into()),
					}
					.into(),
				);
			});
		}
	}
}
