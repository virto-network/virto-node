//! Group issuance and retirement with deposits, as Kreivo wires `communities-manager`
//! (`DEC-12`, `DEC-19`, `DEC-20`, `F-07a`).
//!
//! The values are test values: the deposit is the existential deposit and the delay is 10 days,
//! until `F-07b` sets the measured ones.

use super::*;

use crate::{
	config::{
		communities::{memberships::CommunityMembershipsInstance, Memberships, RetiringMembershipsAccount},
		RelaychainData,
	},
	BlockNumber, RuntimeCall, RuntimeHoldReason, System,
};
use frame_contrib_traits::memberships::InspectEnumerable;
use frame_support::traits::{
	fungible::{hold::Inspect as _, Inspect as _},
	nonfungibles_v2::Inspect as _,
};
use pallet_communities_manager::{Deposits, HoldReason, NextMembershipId, Retiring};
use pallet_nfts::AttributeNamespace;
use runtime_constants::time::DAYS;
use sp_runtime::traits::BlockNumberProvider;

const ALICE: AccountId32 = AccountId32::new([1; 32]);
const BOB: AccountId32 = AccountId32::new([2; 32]);
const CAROL: AccountId32 = AccountId32::new([3; 32]);

const DEPOSIT: u128 = EXISTENTIAL_DEPOSIT;

fn relay_block(n: BlockNumber) {
	System::set_block_number(n);
	RelaychainData::set_block_number(n);
}

/// Community 1, administered by `ALICE`, with funds to hold deposits.
fn community() -> AccountId {
	assert_ok!(CommunitiesManager::register(
		RuntimeOrigin::root(),
		1,
		BoundedVec::try_from(b"First Community".to_vec()).expect("meets max length; qed"),
		CommunityLookup::unlookup(ALICE),
		None,
		None,
	));
	let account = Communities::community_account(&1);
	assert_ok!(Balances::mint_into(&account, 1_000 * DEPOSIT));
	account
}

fn held(account: &AccountId) -> u128 {
	Balances::balance_on_hold(
		&RuntimeHoldReason::CommunitiesManager(HoldReason::MembershipDeposit),
		account,
	)
}

fn stock() -> Vec<u32> {
	let mut stock: Vec<_> = <Memberships as InspectEnumerable<AccountId>>::group_available_memberships(&1).collect();
	stock.sort();
	stock
}

// AC-F1.1, REQ-MI-D1, DEC-31: the community's administrator issues, signing for it or acting as its
// account.
#[test]
fn an_administrator_issues_memberships_into_the_stock_and_holds_their_deposits() {
	TestExternalities::default().execute_with(|| {
		let account = community();

		assert_ok!(CommunitiesManager::issue_memberships(RuntimeOrigin::signed(ALICE), 2));
		assert_ok!(Communities::dispatch_as_account(
			RuntimeOrigin::signed(ALICE),
			Box::new(RuntimeCall::CommunitiesManager(
				pallet_communities_manager::Call::issue_memberships { amount: 1 }
			))
		));

		assert_eq!(stock(), vec![0, 1, 2]);
		assert_eq!(CommunityMemberships::owner(1, 0), Some(account.clone()));
		assert_eq!(held(&account), 3 * DEPOSIT);
		assert_eq!(Deposits::<Runtime>::get((1, 2)), Some(DEPOSIT));
		assert_eq!(NextMembershipId::<Runtime>::get(), 3);

		assert!(CommunitiesManager::issue_memberships(RuntimeOrigin::signed(BOB), 1).is_err());
	})
}

// AC-F2.1, REQ-MI-D1: nothing is issued when the account cannot cover every deposit.
#[test]
fn an_issuance_the_account_cannot_cover_issues_nothing() {
	TestExternalities::default().execute_with(|| {
		let account = community();
		// Leave the account a few deposits, so that one more than it holds is within a call's limit.
		assert_ok!(
			<Balances as frame_support::traits::fungible::Mutate<AccountId>>::transfer(
				&account,
				&BOB,
				990 * DEPOSIT,
				frame_support::traits::tokens::Preservation::Expendable,
			)
		);
		let free = Balances::reducible_balance(
			&account,
			frame_support::traits::tokens::Preservation::Expendable,
			frame_support::traits::tokens::Fortitude::Polite,
		);

		assert_noop!(
			CommunitiesManager::issue_memberships(RuntimeOrigin::signed(ALICE), (free / DEPOSIT) as u32 + 1),
			pallet_communities_manager::Error::<Runtime>::MembershipLimit
		);

		assert!(stock().is_empty());
		assert_eq!(held(&account), 0);
		assert_eq!(NextMembershipId::<Runtime>::get(), 0);
	})
}

// INV-16: an assigned membership carries no allowance, weight or expiration state.
#[test]
fn an_assigned_membership_carries_no_gas_or_expiration_state() {
	TestExternalities::default().execute_with(|| {
		community();
		assert_ok!(CommunitiesManager::issue_memberships(RuntimeOrigin::signed(ALICE), 1));
		assert_ok!(Communities::add_member(
			RuntimeOrigin::signed(ALICE),
			CommunityLookup::unlookup(BOB)
		));

		assert_eq!(CommunityMemberships::owner(1, 0), Some(BOB));
		for key in [b"membership_gas".to_vec(), b"membership_expiration".to_vec()] {
			assert!(CommunityMemberships::system_attribute(&1, Some(&0), &key.encode()).is_none());
		}
	})
}

// AC-F5.1, AC-F5.2, REQ-MI-D2, REQ-MI-4, 0009-A2, m3-retirement-delay: ten days of relay chain
// blocks, and every attribute namespace cleared.
#[test]
fn a_retirement_completes_after_ten_days_of_relay_chain_blocks() {
	TestExternalities::default().execute_with(|| {
		let account = community();
		relay_block(1_000);
		assert_ok!(CommunitiesManager::issue_memberships(RuntimeOrigin::signed(ALICE), 2));
		assert_ok!(CommunitiesManager::retire_memberships(
			RuntimeOrigin::signed(ALICE),
			BoundedVec::try_from(vec![0]).unwrap()
		));

		assert_eq!(stock(), vec![1]);
		assert_eq!(
			CommunityMemberships::owner(1, 0),
			Some(RetiringMembershipsAccount::get())
		);
		assert_eq!(Retiring::<Runtime>::get((1, 0)), Some(1_000 + 10 * DAYS));

		for (namespace, key) in [
			(AttributeNamespace::Pallet, &b"pallet"[..]),
			(AttributeNamespace::ItemOwner, b"owner"),
			(AttributeNamespace::Account(CAROL), b"account"),
		] {
			assert_ok!(CommunityMemberships::force_set_attribute(
				RuntimeOrigin::root(),
				None,
				1,
				Some(0),
				namespace,
				BoundedVec::try_from(key.to_vec()).unwrap(),
				BoundedVec::try_from(b"value".to_vec()).unwrap(),
			));
		}

		// The parachain's own blocks do not count.
		System::set_block_number(1_000_000_000);
		assert_noop!(
			CommunitiesManager::complete_retirement(RuntimeOrigin::signed(CAROL), 1, 0, 0),
			pallet_communities_manager::Error::<Runtime>::RetirementNotElapsed
		);

		relay_block(1_000 + 10 * DAYS);
		assert_ok!(CommunitiesManager::complete_retirement(
			RuntimeOrigin::signed(CAROL),
			1,
			0,
			pallet_nfts::Attribute::<Runtime, CommunityMembershipsInstance>::iter_prefix((1, Some(0))).count() as u32
		));

		assert_eq!(CommunityMemberships::owner(1, 0), None);
		assert_eq!(
			pallet_nfts::Attribute::<Runtime, CommunityMembershipsInstance>::iter_prefix((1, Some(0))).count(),
			0
		);
		assert_eq!(held(&account), DEPOSIT);
		assert_eq!(Deposits::<Runtime>::get((1, 0)), None);
	})
}

// AC-F5.3: an assigned membership is released first.
#[test]
fn an_assigned_membership_is_not_retired() {
	TestExternalities::default().execute_with(|| {
		community();
		assert_ok!(CommunitiesManager::issue_memberships(RuntimeOrigin::signed(ALICE), 1));
		assert_ok!(Communities::add_member(
			RuntimeOrigin::signed(ALICE),
			CommunityLookup::unlookup(BOB)
		));

		assert_noop!(
			CommunitiesManager::retire_memberships(
				RuntimeOrigin::signed(ALICE),
				BoundedVec::try_from(vec![0]).unwrap()
			),
			pallet_communities_manager::Error::<Runtime>::NotRetirable
		);
		assert_eq!(
			<Memberships as frame_contrib_traits::memberships::Inspect<AccountId>>::check_membership(&BOB, &0),
			Some(1)
		);
	})
}

// DEC-38, 0010-Q1, 0010-Q3: the payer pays what registration cost before, split in two.
#[test]
fn a_registration_pays_the_consumption_part_to_the_treasury_and_holds_the_storage_part() {
	use crate::config::communities::{
		CommunityConsumptionPartAmount, CommunityDepositAmount, CommunityStoragePartAmount,
	};

	TestExternalities::default().execute_with(|| {
		let treasury = TreasuryAccount::get();
		assert_ok!(Balances::mint_into(&treasury, EXISTENTIAL_DEPOSIT));
		assert_ok!(Balances::mint_into(&BOB, 10 * UNITS));
		let treasury_before = Balances::balance(&treasury);

		assert_ok!(CommunitiesManager::register(
			RuntimeOrigin::signed(BOB),
			1,
			BoundedVec::try_from(b"First Community".to_vec()).expect("meets max length; qed"),
			CommunityLookup::unlookup(ALICE),
			None,
			None,
		));

		let account = Communities::community_account(&1);
		// The total paid is unchanged: 0.5 KSM.
		assert_eq!(
			CommunityStoragePartAmount::get() + CommunityConsumptionPartAmount::get(),
			CommunityDepositAmount::get()
		);
		assert_eq!(Balances::balance(&BOB), 10 * UNITS - CommunityDepositAmount::get());
		assert_eq!(
			Balances::balance(&treasury),
			treasury_before + CommunityConsumptionPartAmount::get()
		);
		assert_eq!(Balances::total_balance(&account), CommunityStoragePartAmount::get());
		assert_eq!(
			Balances::balance_on_hold(
				&RuntimeHoldReason::CommunitiesManager(HoldReason::CommunityDeposit),
				&account
			),
			CommunityStoragePartAmount::get()
		);
		assert_eq!(
			pallet_communities_manager::CommunityDeposits::<Runtime>::get(1),
			Some(CommunityStoragePartAmount::get())
		);
	})
}

// m3-root-registration: nobody pays.
#[test]
fn a_root_registration_pays_nothing_and_records_nothing() {
	TestExternalities::default().execute_with(|| {
		let treasury_before = Balances::balance(&TreasuryAccount::get());

		let account = community();

		assert_eq!(Balances::balance(&TreasuryAccount::get()), treasury_before);
		assert_eq!(pallet_communities_manager::CommunityDeposits::<Runtime>::get(1), None);
		// Only what the test minted: nothing was held.
		assert_eq!(Balances::total_balance(&account), 1_000 * DEPOSIT);
		assert_eq!(
			Balances::balance_on_hold(
				&RuntimeHoldReason::CommunitiesManager(HoldReason::CommunityDeposit),
				&account
			),
			0
		);
	})
}

// DEC-36: the manager owns and administers the community's collection.
#[test]
fn a_registration_gives_the_collection_to_the_memberships_manager() {
	use crate::config::communities::MembershipsManagerAccount;

	TestExternalities::default().execute_with(|| {
		let account = community();

		let manager = MembershipsManagerAccount::get();
		assert_eq!(CommunityMemberships::collection_owner(1), Some(manager.clone()));
		assert!(pallet_nfts::CollectionRoleOf::<Runtime, CommunityMembershipsInstance>::get(1, &manager).is_some());
		assert!(pallet_nfts::CollectionRoleOf::<Runtime, CommunityMembershipsInstance>::get(1, &account).is_none());
	})
}

/// The benchmarks of the group calls run in Kreivo's runtime: they stop on any setup the runtime
/// cannot satisfy, where a skipped (weightless) benchmark would leave its call without a weight.
#[cfg(feature = "runtime-benchmarks")]
#[test]
fn group_benchmarks_run_in_the_runtime() {
	for name in ["issue_memberships", "retire_memberships", "complete_retirement"] {
		TestExternalities::default().execute_with(|| {
			relay_block(1);
			assert_ok!(pallet_communities_manager::test_group_benchmark::<crate::Runtime>(name));
		});
	}
}
