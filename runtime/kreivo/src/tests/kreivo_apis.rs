//! The Kreivo APIs chain extension, called by a contract deployed on the
//! runtime.
//!
//! The contract under test is the `kreivo_apis_proxy` fixture: it forwards its
//! input to the chain extension (or to another contract), and returns the
//! chain extension's return code and output untouched.

use super::*;

use crate::{
	config::contracts::{KreivoApisBurnWeight, KreivoApisMintWeight},
	Assets, AssetsFreezer, Contracts, RuntimeFreezeReason,
};
use ::kreivo_apis::apis::{AssetsApiError, KreivoApisError, KreivoApisErrorCode};
use frame_support::{
	assert_noop,
	pallet_prelude::{DispatchError, Weight},
	storage::{with_transaction, TransactionOutcome},
	traits::fungibles::{freeze::Mutate as _, Inspect as _},
};
use pallet_contracts::{Code, CollectEvents, DebugInfo, Determinism};
use parachains_common::Balance;
use parity_scale_codec::Decode;
use virto_common::CommunityId;

const DEPLOYER: AccountId = AccountId::new([0xde; 32]);
const ALICE: AccountId = AccountId::new([1; 32]);
const BOB: AccountId = AccountId::new([2; 32]);
const FREEZER: AccountId = AccountId::new([0xf0; 32]);

const COMMUNITY: CommunityId = 1;
const OTHER_COMMUNITY: CommunityId = 2;

/// Asset 101 belongs to `COMMUNITY`, asset 102 to `OTHER_COMMUNITY`.
const ASSET: FungibleAssetLocation = FungibleAssetLocation::Here(101);
const OTHER_ASSET: FungibleAssetLocation = FungibleAssetLocation::Here(102);
const MIN_BALANCE: Balance = 10;

const BALANCE: u32 = 0x0000;
const MINT: u32 = 0x0003;
const BURN: u32 = 0x0004;

fn gas_limit() -> Weight {
	Weight::from_parts(500_000_000_000, 4 * 1024 * 1024)
}

fn community_account(community: CommunityId) -> AccountId {
	Communities::community_account(&community)
}

/// Deploys the proxy contract. If a `community` is given, it is registered as
/// an instance of that community, the way `ContractsStore::instantiate` does.
fn deploy(salt: u8, community: Option<CommunityId>) -> AccountId {
	let (code, _) =
		pallet_contracts_fixtures::compile_module::<Runtime>("kreivo_apis_proxy").expect("the fixture is built; qed");
	let contract = Contracts::bare_instantiate(
		DEPLOYER,
		0,
		gas_limit(),
		None,
		Code::Upload(code),
		vec![],
		vec![salt],
		DebugInfo::Skip,
		CollectEvents::Skip,
	)
	.result
	.expect("the proxy deploys; qed")
	.account_id;

	if let Some(community) = community {
		pallet_contracts_store::ContractMerchantId::<Runtime>::insert(&contract, community);
	}
	contract
}

/// The raw output of calling the chain extension `func_id` through `contract`:
/// the return code, the output, and the gas consumed by the whole call.
fn raw_call(contract: &AccountId, func_id: u32, input: impl Encode) -> (u32, Vec<u8>, Weight) {
	let mut data = vec![0u8];
	data.extend(func_id.to_le_bytes());
	data.extend(input.encode());
	call_contract(contract, data)
}

fn call_contract(contract: &AccountId, data: Vec<u8>) -> (u32, Vec<u8>, Weight) {
	let result = Contracts::bare_call(
		DEPLOYER,
		contract.clone(),
		0,
		gas_limit(),
		None,
		data,
		DebugInfo::Skip,
		CollectEvents::Skip,
		Determinism::Enforced,
	);
	let output = result.result.expect("the contract does not trap; qed");
	assert!(!output.did_revert());
	let ret_code = u32::from_le_bytes(output.data[..4].try_into().expect("4 bytes; qed"));
	(ret_code, output.data[4..].to_vec(), result.gas_consumed)
}

fn decode_result(ret_code: u32, output: Vec<u8>) -> Result<Balance, KreivoApisError> {
	if ret_code == 0 {
		Ok(Balance::decode(&mut &output[..]).expect("the chain extension returns a balance; qed"))
	} else {
		Err(KreivoApisErrorCode::from(ret_code).into())
	}
}

fn mint(
	contract: &AccountId,
	asset: FungibleAssetLocation,
	amount: Balance,
	beneficiary: &AccountId,
) -> Result<Balance, KreivoApisError> {
	let (ret_code, output, _) = raw_call(contract, MINT, (asset, amount, beneficiary));
	decode_result(ret_code, output)
}

fn burn(
	contract: &AccountId,
	asset: FungibleAssetLocation,
	amount: Balance,
	who: &AccountId,
) -> Result<Balance, KreivoApisError> {
	let (ret_code, output, _) = raw_call(contract, BURN, (asset, amount, who));
	decode_result(ret_code, output)
}

/// What `pallet_assets::mint`, signed by `COMMUNITY`'s account, does in the
/// current state. Nothing is kept.
fn mint_extrinsic(
	asset: FungibleAssetLocation,
	amount: Balance,
	beneficiary: &AccountId,
) -> Result<Balance, DispatchError> {
	dry_run(|| {
		let before = Assets::balance(asset, beneficiary);
		Assets::mint(
			RuntimeOrigin::signed(community_account(COMMUNITY)),
			asset,
			CommunityLookup::unlookup(beneficiary.clone()),
			amount,
		)?;
		Ok(Assets::balance(asset, beneficiary) - before)
	})
}

/// What `pallet_assets::burn`, signed by `COMMUNITY`'s account, does in the
/// current state. Nothing is kept.
fn burn_extrinsic(asset: FungibleAssetLocation, amount: Balance, who: &AccountId) -> Result<Balance, DispatchError> {
	dry_run(|| {
		let before = Assets::balance(asset, who);
		Assets::burn(
			RuntimeOrigin::signed(community_account(COMMUNITY)),
			asset,
			CommunityLookup::unlookup(who.clone()),
			amount,
		)?;
		Ok(before - Assets::balance(asset, who))
	})
}

fn dry_run<R>(f: impl FnOnce() -> Result<R, DispatchError>) -> Result<R, DispatchError> {
	with_transaction(|| TransactionOutcome::Rollback(Ok::<_, DispatchError>(f())))
		.expect("the transaction layer works; qed")
}

/// Creates `asset`, owned by `community`, with `issuer` and `admin` as its team.
fn create_asset(asset: FungibleAssetLocation, community: CommunityId, issuer: &AccountId, admin: &AccountId) {
	let owner = community_account(community);
	assert_ok!(Assets::force_create(
		RuntimeOrigin::root(),
		asset,
		CommunityLookup::unlookup(owner.clone()),
		true,
		MIN_BALANCE,
	));
	assert_ok!(Assets::set_team(
		RuntimeOrigin::signed(owner),
		asset,
		CommunityLookup::unlookup(issuer.clone()),
		CommunityLookup::unlookup(admin.clone()),
		CommunityLookup::unlookup(FREEZER),
	));
}

/// `ASSET` (issued and administered by `COMMUNITY`), `OTHER_ASSET` (by
/// `OTHER_COMMUNITY`), and `BOB` holding 100 of `ASSET`.
fn new_test_ext() -> TestExternalities {
	let mut ext = TestExternalities::default();
	ext.execute_with(|| {
		assert_ok!(Balances::mint_into(&DEPLOYER, 1_000 * UNITS));
		assert_ok!(Balances::mint_into(&FREEZER, UNITS));
		for community in [COMMUNITY, OTHER_COMMUNITY] {
			assert_ok!(Balances::mint_into(&community_account(community), UNITS));
		}

		let account = community_account(COMMUNITY);
		create_asset(ASSET, COMMUNITY, &account, &account);
		let other_account = community_account(OTHER_COMMUNITY);
		create_asset(OTHER_ASSET, OTHER_COMMUNITY, &other_account, &other_account);

		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(account),
			ASSET,
			CommunityLookup::unlookup(BOB),
			100
		));
	});
	ext
}

fn not_authorised() -> Result<Balance, KreivoApisError> {
	Err(AssetsApiError::NotAuthorised.into())
}

fn cannot_mint() -> Result<Balance, KreivoApisError> {
	Err(AssetsApiError::CannotMint.into())
}

fn cannot_burn() -> Result<Balance, KreivoApisError> {
	Err(AssetsApiError::CannotBurn.into())
}

#[test]
fn mint_works_when_the_community_is_the_issuer() {
	new_test_ext().execute_with(|| {
		let contract = deploy(0, Some(COMMUNITY));
		assert_eq!(mint_extrinsic(ASSET, 50, &ALICE), Ok(50));

		assert_eq!(mint(&contract, ASSET, 50, &ALICE), Ok(50));
		assert_eq!(Assets::balance(ASSET, &ALICE), 50);
		assert_eq!(Assets::total_issuance(ASSET), 150);
	})
}

#[test]
fn mint_is_refused_when_the_community_is_only_owner_or_admin() {
	new_test_ext().execute_with(|| {
		let account = community_account(COMMUNITY);
		// The community owns and administers the asset, but `ALICE` issues it.
		assert_ok!(Assets::set_team(
			RuntimeOrigin::signed(account.clone()),
			ASSET,
			CommunityLookup::unlookup(ALICE),
			CommunityLookup::unlookup(account),
			CommunityLookup::unlookup(FREEZER),
		));
		let contract = deploy(0, Some(COMMUNITY));

		assert_eq!(mint(&contract, ASSET, 50, &BOB), not_authorised());
		assert_eq!(Assets::balance(ASSET, &BOB), 100);
		assert_eq!(Assets::total_issuance(ASSET), 100);
	})
}

#[test]
fn mint_is_refused_for_another_communitys_asset() {
	new_test_ext().execute_with(|| {
		let contract = deploy(0, Some(COMMUNITY));

		assert_eq!(mint(&contract, OTHER_ASSET, 50, &BOB), not_authorised());
		assert_eq!(burn(&contract, OTHER_ASSET, 50, &BOB), not_authorised());
		assert_eq!(Assets::total_issuance(OTHER_ASSET), 0);
	})
}

#[test]
fn mint_and_burn_are_refused_for_a_non_local_asset_even_with_the_roles() {
	new_test_ext().execute_with(|| {
		let contract = deploy(0, Some(COMMUNITY));
		let account = community_account(COMMUNITY);
		let sibling = FungibleAssetLocation::Sibling(virto_common::Para {
			id: 1000,
			pallet: 50,
			index: 1984,
		});
		let external = FungibleAssetLocation::External {
			network: virto_common::NetworkId::Polkadot,
			child: None,
		};

		for asset in [sibling, external] {
			// Somehow, the community issues and administers the asset.
			create_asset(asset, COMMUNITY, &account, &account);
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(account.clone()),
				asset,
				CommunityLookup::unlookup(BOB),
				100
			));

			assert_eq!(
				mint(&contract, asset, 50, &BOB),
				Err(AssetsApiError::NotALocalAsset.into())
			);
			assert_eq!(
				burn(&contract, asset, 50, &BOB),
				Err(AssetsApiError::NotALocalAsset.into())
			);
			assert_eq!(Assets::balance(asset, &BOB), 100);
		}

		// A local asset still works.
		assert_eq!(mint(&contract, ASSET, 50, &BOB), Ok(50));
		assert_eq!(burn(&contract, ASSET, 50, &BOB), Ok(50));
	})
}

#[test]
fn mint_and_burn_are_refused_for_a_contract_outside_the_store() {
	new_test_ext().execute_with(|| {
		let contract = deploy(0, None);

		assert_eq!(
			mint(&contract, ASSET, 50, &BOB),
			Err(AssetsApiError::NotAGroupContract.into())
		);
		assert_eq!(
			burn(&contract, ASSET, 50, &BOB),
			Err(AssetsApiError::NotAGroupContract.into())
		);
		assert_eq!(Assets::balance(ASSET, &BOB), 100);
	})
}

#[test]
fn burn_works_as_admin_from_a_third_party_account() {
	new_test_ext().execute_with(|| {
		let contract = deploy(0, Some(COMMUNITY));
		assert_eq!(burn_extrinsic(ASSET, 30, &BOB), Ok(30));

		assert_eq!(burn(&contract, ASSET, 30, &BOB), Ok(30));
		assert_eq!(Assets::balance(ASSET, &BOB), 70);
		assert_eq!(Assets::total_issuance(ASSET), 70);
	})
}

#[test]
fn burn_is_best_effort_and_reaps_the_account() {
	new_test_ext().execute_with(|| {
		let contract = deploy(0, Some(COMMUNITY));
		assert_eq!(burn_extrinsic(ASSET, 1_000, &BOB), Ok(100));

		assert_eq!(burn(&contract, ASSET, 1_000, &BOB), Ok(100));
		assert_eq!(Assets::balance(ASSET, &BOB), 0);
		assert_eq!(Assets::total_issuance(ASSET), 0);
	})
}

#[test]
fn burn_is_refused_when_the_community_is_only_issuer() {
	new_test_ext().execute_with(|| {
		let account = community_account(COMMUNITY);
		// The community owns and issues the asset, but `ALICE` administers it.
		assert_ok!(Assets::set_team(
			RuntimeOrigin::signed(account.clone()),
			ASSET,
			CommunityLookup::unlookup(account),
			CommunityLookup::unlookup(ALICE),
			CommunityLookup::unlookup(FREEZER),
		));
		let contract = deploy(0, Some(COMMUNITY));

		assert_eq!(burn(&contract, ASSET, 30, &BOB), not_authorised());
		assert_eq!(Assets::balance(ASSET, &BOB), 100);
	})
}

#[test]
fn mint_respects_the_minimum_balance() {
	new_test_ext().execute_with(|| {
		let contract = deploy(0, Some(COMMUNITY));

		// Below the minimum balance, a new account can't be created...
		assert!(mint_extrinsic(ASSET, MIN_BALANCE - 1, &ALICE).is_err());
		assert_eq!(mint(&contract, ASSET, MIN_BALANCE - 1, &ALICE), cannot_mint());
		assert_eq!(Assets::balance(ASSET, &ALICE), 0);

		// ...but an existing one can receive any amount.
		assert_eq!(mint_extrinsic(ASSET, 1, &BOB), Ok(1));
		assert_eq!(mint(&contract, ASSET, 1, &BOB), Ok(1));
		assert_eq!(mint(&contract, ASSET, MIN_BALANCE, &ALICE), Ok(MIN_BALANCE));
	})
}

#[test]
fn burn_takes_the_dust_below_the_minimum_balance() {
	new_test_ext().execute_with(|| {
		let contract = deploy(0, Some(COMMUNITY));
		// Leaving 5 (< `MIN_BALANCE`) would leave dust, so the whole balance is burnt.
		assert_eq!(burn_extrinsic(ASSET, 95, &BOB), Ok(100));

		assert_eq!(burn(&contract, ASSET, 95, &BOB), Ok(100));
		assert_eq!(Assets::balance(ASSET, &BOB), 0);
		assert_eq!(Assets::total_issuance(ASSET), 0);
	})
}

#[test]
fn a_frozen_asset_can_be_neither_minted_nor_burnt() {
	new_test_ext().execute_with(|| {
		let contract = deploy(0, Some(COMMUNITY));
		assert_ok!(Assets::freeze_asset(RuntimeOrigin::signed(FREEZER), ASSET));

		assert!(mint_extrinsic(ASSET, 50, &BOB).is_err());
		assert_eq!(mint(&contract, ASSET, 50, &BOB), cannot_mint());

		assert!(burn_extrinsic(ASSET, 50, &BOB).is_err());
		assert_eq!(burn(&contract, ASSET, 50, &BOB), cannot_burn());

		// Like the extrinsic, burning nothing is a no-op that goes through.
		assert_eq!(burn_extrinsic(ASSET, 0, &BOB), Ok(0));
		assert_eq!(burn(&contract, ASSET, 0, &BOB), Ok(0));

		assert_eq!(Assets::balance(ASSET, &BOB), 100);
		assert_eq!(Assets::total_issuance(ASSET), 100);
	})
}

#[test]
fn an_asset_being_destroyed_can_be_neither_minted_nor_burnt() {
	new_test_ext().execute_with(|| {
		let contract = deploy(0, Some(COMMUNITY));
		assert_ok!(Assets::start_destroy(
			RuntimeOrigin::signed(community_account(COMMUNITY)),
			ASSET
		));

		assert!(mint_extrinsic(ASSET, 50, &BOB).is_err());
		assert_eq!(mint(&contract, ASSET, 50, &BOB), cannot_mint());

		assert!(burn_extrinsic(ASSET, 50, &BOB).is_err());
		assert_eq!(burn(&contract, ASSET, 50, &BOB), cannot_burn());
		assert!(burn_extrinsic(ASSET, 0, &BOB).is_err());
		assert_eq!(burn(&contract, ASSET, 0, &BOB), cannot_burn());

		assert_eq!(Assets::balance(ASSET, &BOB), 100);
	})
}

#[test]
fn a_frozen_account_can_receive_but_cannot_be_burnt_from() {
	new_test_ext().execute_with(|| {
		let contract = deploy(0, Some(COMMUNITY));
		assert_ok!(Assets::freeze(
			RuntimeOrigin::signed(FREEZER),
			ASSET,
			CommunityLookup::unlookup(BOB)
		));

		// `pallet_assets` lets a frozen account receive.
		assert_eq!(mint_extrinsic(ASSET, 50, &BOB), Ok(50));
		assert_eq!(mint(&contract, ASSET, 50, &BOB), Ok(50));

		assert!(burn_extrinsic(ASSET, 50, &BOB).is_err());
		assert_eq!(burn(&contract, ASSET, 50, &BOB), cannot_burn());
		assert_eq!(Assets::balance(ASSET, &BOB), 150);
	})
}

#[test]
fn a_blocked_account_can_neither_receive_nor_be_burnt_from() {
	new_test_ext().execute_with(|| {
		let contract = deploy(0, Some(COMMUNITY));
		assert_ok!(Assets::block(
			RuntimeOrigin::signed(FREEZER),
			ASSET,
			CommunityLookup::unlookup(BOB)
		));

		assert!(mint_extrinsic(ASSET, 50, &BOB).is_err());
		assert_eq!(mint(&contract, ASSET, 50, &BOB), cannot_mint());

		assert!(burn_extrinsic(ASSET, 50, &BOB).is_err());
		assert_eq!(burn(&contract, ASSET, 50, &BOB), cannot_burn());
		assert_eq!(Assets::balance(ASSET, &BOB), 100);
	})
}

#[test]
fn burn_from_an_account_without_the_asset_is_refused() {
	new_test_ext().execute_with(|| {
		let contract = deploy(0, Some(COMMUNITY));

		assert!(burn_extrinsic(ASSET, 50, &ALICE).is_err());
		assert_eq!(burn(&contract, ASSET, 50, &ALICE), cannot_burn());
	})
}

#[test]
fn burn_leaves_frozen_funds_alone() {
	new_test_ext().execute_with(|| {
		let contract = deploy(0, Some(COMMUNITY));
		// A vote locks 60 of `BOB`'s 100.
		assert_ok!(AssetsFreezer::set_freeze(
			ASSET,
			&RuntimeFreezeReason::Communities(pallet_communities::FreezeReason::VoteCasted),
			&BOB,
			60,
		));
		assert_eq!(burn_extrinsic(ASSET, 100, &BOB), Ok(40));

		assert_eq!(burn(&contract, ASSET, 100, &BOB), Ok(40));
		assert_eq!(Assets::balance(ASSET, &BOB), 60);
		assert_eq!(Assets::total_issuance(ASSET), 60);
	})
}

#[test]
fn mint_and_burn_charge_their_weight_up_front() {
	new_test_ext().execute_with(|| {
		let contract = deploy(0, Some(COMMUNITY));
		let stranger = deploy(1, None);

		// A balance query charges no weight of its own: it is the baseline.
		let (_, _, baseline) = raw_call(&contract, BALANCE, (ASSET, BOB));

		let charged = |contract: &AccountId, func_id: u32, amount: Balance| {
			let (_, _, consumed) = raw_call(contract, func_id, (ASSET, amount, BOB));
			consumed.saturating_sub(baseline)
		};

		// The weight is charged whether the call succeeds or is refused, and
		// copying a few more input bytes than the baseline costs little.
		let close_to = |actual: Weight, expected: Weight| {
			actual.all_gte(expected)
				&& expected
					.saturating_add(Weight::from_parts(50_000_000, 0))
					.all_gte(actual)
		};
		for (contract, amount) in [(&contract, 10), (&stranger, 10)] {
			assert!(close_to(charged(contract, MINT, amount), KreivoApisMintWeight::get()));
			assert!(close_to(charged(contract, BURN, amount), KreivoApisBurnWeight::get()));
		}

		// Running out of gas while charging traps the contract: nothing happens.
		let result = Contracts::bare_call(
			DEPLOYER,
			contract.clone(),
			0,
			baseline.saturating_add(Weight::from_parts(KreivoApisMintWeight::get().ref_time() / 2, 0)),
			None,
			[
				vec![0u8],
				MINT.to_le_bytes().to_vec(),
				(ASSET, 10 as Balance, BOB).encode(),
			]
			.concat(),
			DebugInfo::Skip,
			CollectEvents::Skip,
			Determinism::Enforced,
		);
		assert!(result.result.is_err());
		assert_eq!(Assets::balance(ASSET, &BOB), 100);
	})
}

#[test]
fn a_contract_called_by_another_authorises_as_itself_not_as_its_caller() {
	new_test_ext().execute_with(|| {
		let ours = deploy(0, Some(COMMUNITY));
		let theirs = deploy(1, Some(OTHER_COMMUNITY));

		let via = |caller: &AccountId, callee: &AccountId, asset: FungibleAssetLocation| {
			let mut data = vec![1u8];
			data.extend(callee.encode());
			data.push(0);
			data.extend(MINT.to_le_bytes());
			data.extend((asset, 50 as Balance, ALICE).encode());
			let (ret_code, output, _) = call_contract(caller, data);
			decode_result(ret_code, output)
		};

		// `theirs` can't mint `ASSET` because it is called by `ours`...
		assert_eq!(via(&ours, &theirs, ASSET), not_authorised());
		// ...but it mints `OTHER_ASSET`, whoever calls it.
		assert_eq!(via(&ours, &theirs, OTHER_ASSET), Ok(50));
		assert_eq!(Assets::balance(OTHER_ASSET, &ALICE), 50);
		// And the other way around.
		assert_eq!(via(&theirs, &ours, OTHER_ASSET), not_authorised());
		assert_eq!(via(&theirs, &ours, ASSET), Ok(50));
		assert_eq!(Assets::balance(ASSET, &ALICE), 50);
	})
}

#[test]
fn the_extrinsic_helpers_keep_nothing() {
	new_test_ext().execute_with(|| {
		assert_eq!(burn_extrinsic(ASSET, 30, &BOB), Ok(30));
		assert_eq!(mint_extrinsic(ASSET, 30, &BOB), Ok(30));
		assert_eq!(Assets::balance(ASSET, &BOB), 100);
		assert_noop!(
			Assets::mint(RuntimeOrigin::signed(ALICE), ASSET, CommunityLookup::unlookup(BOB), 30),
			pallet_assets::Error::<Runtime, crate::config::currency::KreivoAssetsInstance>::NoPermission
		);
	})
}
