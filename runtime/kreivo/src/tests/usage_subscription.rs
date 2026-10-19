//! Usage subscriptions as Kreivo wires them (`F-08`): the origins and tracks, the usable groups,
//! the payment step in the transaction extensions, and a KSM charge to the Treasury.
//!
//! The values are test values: `F-07b` sets the measured ones.

use super::*;
use frame_support::weights::Weight;

use crate::{
	config::{
		collective::{
			tracks::{TrackId, TracksInfo},
			Origin as CustomOrigin,
		},
		payments::KreivoFeeHandler,
		usage_subscription::ActiveCommunity,
	},
	BlockNumber, ChargeTransaction, RuntimeCall, RuntimeEvent, System, UsageSubscription, DAYS, HOURS,
};
use frame_support::{
	dispatch::{DispatchInfo, GetDispatchInfo},
	traits::{
		fungible::{hold::Inspect as _, Inspect as _},
		Contains, OnRuntimeUpgrade,
	},
};
use frame_system::{CheckNonce, CheckWeight};
use pallet_asset_tx_payment::ChargeAssetTxPayment;
use pallet_communities::types::CommunityState;
use pallet_payments::types::FeeHandler;
use pallet_referenda::TracksInfo as _;
use pallet_skip_feeless_payment::SkipCheckIfFeeless;
use pallet_usage_subscription::{Event, OfferKind, Terms};
use scale_info::TypeInfo;
use sp_runtime::{
	generic::ExtrinsicFormat,
	traits::{Applyable, TransactionExtension},
	Perbill,
};

const ALICE: AccountId32 = AccountId32::new([1; 32]);
const BOB: AccountId32 = AccountId32::new([2; 32]);

/// The price of one billing period, in KSM.
const PRICE: u128 = UNITS / 100;

fn relay_block(n: BlockNumber) {
	System::set_block_number(n);
	<crate::config::RelaychainData as sp_runtime::traits::BlockNumberProvider>::set_block_number(n);
}

/// Community 1, administered by `ALICE`, with funds.
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
	assert_ok!(Balances::mint_into(&account, 1_000 * UNITS));
	account
}

fn admin() -> RuntimeOrigin {
	CustomOrigin::UsageSubscriptionAdmin.into()
}

fn amender() -> RuntimeOrigin {
	CustomOrigin::UsageSubscriptionAmender.into()
}

fn terms() -> Terms<pallet_listings::ItemPrice<FungibleAssetLocation, u128>, BlockNumber> {
	Terms {
		allowance: Weight::from_parts(1_000_000_000_000, 10_000_000),
		usage_period: HOURS,
		price: pallet_listings::ItemPrice {
			asset: FungibleAssetLocation::Here(0),
			amount: PRICE,
		},
		billing_period: 30 * DAYS,
		term: None,
		min_commitment: None,
		grace: 3 * DAYS,
	}
}

/// Community 1 subscribes to a standard offer, and `BOB` is one of its members, with no balance
/// and a provider reference only, as a wallet-less pass account.
fn subscribed_group_with_a_member() -> AccountId {
	let account = community();
	relay_block(1_000);
	assert_ok!(Balances::mint_into(&TreasuryAccount::get(), 1_000 * UNITS));
	assert_ok!(UsageSubscription::publish_offer(admin(), OfferKind::Standard, terms()));
	assert_ok!(UsageSubscription::subscribe(
		RuntimeOrigin::signed(account.clone()),
		0,
		None
	));

	assert_ok!(CommunitiesManager::issue_memberships(RuntimeOrigin::signed(ALICE), 1));
	assert_ok!(Communities::add_member(
		RuntimeOrigin::signed(ALICE),
		CommunityLookup::unlookup(BOB)
	));
	System::inc_providers(&BOB);
	assert_eq!(Balances::balance(&BOB), 0);
	account
}

type PaymentStep = (
	CheckNonce<Runtime>,
	CheckWeight<Runtime>,
	SkipCheckIfFeeless<Runtime, ChargeTransaction>,
);

fn payment_step() -> PaymentStep {
	(
		CheckNonce::from(0),
		CheckWeight::new(),
		SkipCheckIfFeeless::from(ChargeTransaction::from(ChargeAssetTxPayment::from(0, None))),
	)
}

/// Applies `call` signed by `who` through the nonce, weight and payment extensions.
fn apply(
	who: &AccountId,
	call: RuntimeCall,
) -> sp_runtime::ApplyExtrinsicResultWithInfo<frame_support::dispatch::PostDispatchInfo> {
	let ext = payment_step();
	let info = DispatchInfo {
		extension_weight: ext.weight(&call),
		..call.get_dispatch_info()
	};
	let len = call.encoded_size();
	sp_runtime::generic::CheckedExtrinsic::<AccountId, RuntimeCall, PaymentStep> {
		format: ExtrinsicFormat::Signed(who.clone(), ext),
		function: call,
	}
	.apply::<Runtime>(&info, len)
}

// DEC-10: the origin keeps variant 0 and its track, and the amender is variant 4 on track 5.
#[test]
fn the_admin_origin_keeps_its_encoding_and_track_and_the_amender_is_added() {
	assert_eq!(CustomOrigin::UsageSubscriptionAdmin.encode(), vec![0]);
	assert_eq!(CustomOrigin::UsageSubscriptionAmender.encode(), vec![4]);

	let track = |origin: CustomOrigin| {
		let origin: RuntimeOrigin = origin.into();
		TracksInfo::track_for(&origin.caller).expect("the origin has a track")
	};
	assert_eq!(track(CustomOrigin::UsageSubscriptionAdmin), 3);
	assert_eq!(track(CustomOrigin::UsageSubscriptionAmender), 5);

	let name_of = |id: TrackId| {
		TracksInfo::tracks()
			.find(|t| t.id == id)
			.map(|t| t.info.name)
			.expect("the track exists")
	};
	assert_eq!(name_of(3), sp_runtime::str_array::<25>("Usage Subscriptions"));
}

// DEC-28, deployment document section 4: track 5's values.
#[test]
fn the_amendment_track_has_the_deployment_values() {
	let track = TracksInfo::tracks().find(|t| t.id == 5).expect("the track exists");
	let info = &track.info;
	assert_eq!(info.prepare_period, DAYS);
	assert_eq!(info.decision_period, 28 * DAYS);
	assert_eq!(info.confirm_period, 14 * DAYS);
	assert_eq!(info.min_enactment_period, 7 * DAYS);
	assert_eq!(
		info.min_approval,
		pallet_referenda::Curve::make_reciprocal(
			3,
			28,
			sp_runtime::FixedI64::from_rational(75, 100),
			sp_runtime::FixedI64::from_rational(60, 100),
			sp_runtime::FixedI64::from_rational(100, 100),
		)
	);
	assert_eq!(
		info.min_support,
		pallet_referenda::Curve::make_linear(
			28,
			28,
			sp_runtime::FixedI64::from_rational(60, 100),
			sp_runtime::FixedI64::from_rational(60, 100),
		)
	);
}

// REQ-GR-4: only an active community that is not the collective is usable.
#[test]
fn community_zero_and_blocked_communities_are_not_usable() {
	TestExternalities::default().execute_with(|| {
		community();
		assert!(ActiveCommunity::contains(&1));
		assert!(!ActiveCommunity::contains(&0));
		assert!(!ActiveCommunity::contains(&7));

		Communities::force_state(&1, CommunityState::Blocked);
		assert!(!ActiveCommunity::contains(&1));
	})
}

// DEC-24: the payment step is invisible in the metadata and the extrinsic bytes.
#[test]
fn the_payment_step_leaves_the_metadata_and_the_bytes_of_the_fee_extension() {
	type Fee = ChargeAssetTxPayment<Runtime>;
	assert_eq!(
		<ChargeTransaction as TypeInfo>::type_info().type_def,
		<Fee as TypeInfo>::type_info().type_def
	);
	let fee = ChargeAssetTxPayment::<Runtime>::from(7, None);
	assert_eq!(ChargeTransaction::from(fee.clone()).encode(), fee.encode());
	assert_eq!(
		<ChargeTransaction as TransactionExtension<RuntimeCall>>::metadata()[0].identifier,
		<Fee as TransactionExtension<RuntimeCall>>::metadata()[0].identifier
	);
}

// DEC-36's counterpart: GasTxPayment is gone, and what it left under its prefix is cleared.
#[test]
fn the_removed_gas_pallet_leaves_no_keys() {
	TestExternalities::default().execute_with(|| {
		let prefix = sp_io::hashing::twox_128(b"GasTxPayment");
		let key = [
			prefix.to_vec(),
			sp_io::hashing::twox_128(b"Tanks").to_vec(),
			vec![1, 2, 3],
		]
		.concat();
		sp_io::storage::set(&key, &[1]);

		<frame_support::migrations::RemovePallet<
			crate::GasTxPaymentPrefix,
			frame_support::weights::constants::RocksDbWeight,
		> as OnRuntimeUpgrade>::on_runtime_upgrade();

		assert!(sp_io::storage::next_key(&prefix).is_none_or(|next| !next.starts_with(&prefix)));
	})
}

// DEC-5, REQ-BL-3: a charge to the Treasury pays no fee on either side; any other beneficiary does.
#[test]
fn a_payment_to_the_treasury_pays_no_fee() {
	TestExternalities::default().execute_with(|| {
		let native = FungibleAssetLocation::Here(0);
		let to_treasury = KreivoFeeHandler::apply_fees(&native, &ALICE, &TreasuryAccount::get(), &PRICE, None);
		assert!(to_treasury.sender_pays.is_empty());
		assert!(to_treasury.beneficiary_pays.is_empty());

		let to_bob = KreivoFeeHandler::apply_fees(&native, &ALICE, &BOB, &PRICE, None);
		assert!(!to_bob.sender_pays.is_empty());
		assert!(!to_bob.beneficiary_pays.is_empty());
	})
}

// DEC-11, REQ-BL-3: a KSM charge moves exactly the price from the group to the Treasury, and the
// subscription's deposit is held on the group.
#[test]
fn a_ksm_charge_moves_exactly_the_price_to_the_treasury() {
	TestExternalities::default().execute_with(|| {
		let account = community();
		relay_block(1_000);
		assert_ok!(Balances::mint_into(&TreasuryAccount::get(), 1_000 * UNITS));
		assert_ok!(UsageSubscription::publish_offer(admin(), OfferKind::Standard, terms()));

		let treasury = Balances::balance(&TreasuryAccount::get());
		let group = Balances::total_balance(&account);
		assert_ok!(UsageSubscription::subscribe(
			RuntimeOrigin::signed(account.clone()),
			0,
			None
		));

		assert_eq!(Balances::balance(&TreasuryAccount::get()), treasury + PRICE);
		let held = Balances::total_balance_on_hold(&account);
		assert_eq!(Balances::total_balance(&account), group - PRICE);
		assert!(held > 0, "the subscription deposit is held on the group");
	})
}

// AC-C3.3, 0009-A12: a member with no balance transacts on its group's pool, through the real
// nonce, weight and payment extensions.
#[test]
fn a_member_with_no_balance_transacts_on_the_pool_of_its_group() {
	TestExternalities::default().execute_with(|| {
		subscribed_group_with_a_member();

		let remark = RuntimeCall::System(frame_system::Call::remark { remark: vec![1] });
		assert_ok!(apply(&BOB, remark));

		assert_eq!(Balances::balance(&BOB), 0);
		assert!(System::events().iter().any(|e| matches!(
			e.event,
			RuntimeEvent::UsageSubscription(Event::UsageCharged { group: 1, .. })
		)));
	})
}

// A non-member with no balance cannot pay the fee: the pool is not theirs (REQ-PL-7).
#[test]
fn an_account_outside_the_group_with_no_balance_cannot_transact() {
	TestExternalities::default().execute_with(|| {
		subscribed_group_with_a_member();
		let carol = AccountId32::new([3; 32]);
		System::inc_providers(&carol);

		let remark = RuntimeCall::System(frame_system::Call::remark { remark: vec![1] });
		assert!(apply(&carol, remark).is_err());
	})
}

// m3-capacity-origin, DEC-10, DEC-28: Root and track 3 publish and cap; the amender only amends.
#[test]
fn offers_and_capacity_answer_to_root_and_track_three_and_amendments_to_track_five() {
	TestExternalities::default().execute_with(|| {
		community();
		relay_block(1_000);
		assert_ok!(Balances::mint_into(&TreasuryAccount::get(), 1_000 * UNITS));

		let capacity = Some(Weight::from_parts(1_000_000_000_000_000, 1_000_000_000));
		assert!(UsageSubscription::set_capacity(RuntimeOrigin::signed(ALICE), capacity, None).is_err());
		assert!(UsageSubscription::set_capacity(amender(), capacity, None).is_err());
		assert_ok!(UsageSubscription::set_capacity(admin(), capacity, None));
		assert_ok!(UsageSubscription::set_capacity(RuntimeOrigin::root(), None, None));

		assert!(UsageSubscription::publish_offer(amender(), OfferKind::Standard, terms()).is_err());
		assert_ok!(UsageSubscription::publish_offer(
			RuntimeOrigin::root(),
			OfferKind::Standard,
			terms()
		));
		assert_ok!(UsageSubscription::publish_offer(
			admin(),
			OfferKind::Trial,
			Terms {
				term: Some(1),
				..terms()
			}
		));

		assert!(UsageSubscription::amend_offer(admin(), 0, terms()).is_err());
		assert!(UsageSubscription::amend_offer(RuntimeOrigin::signed(ALICE), 0, terms()).is_err());
	})
}

// m3-alert-threshold, m3-capacity-origin: the capacity values.
#[test]
fn the_capacity_values_are_those_of_the_deployment_document() {
	use crate::config::usage_subscription::{CapacityAlertThreshold, CapacityPeriod};
	assert_eq!(CapacityAlertThreshold::get(), Perbill::from_percent(80));
	assert_eq!(CapacityPeriod::get(), 7 * DAYS);
}
