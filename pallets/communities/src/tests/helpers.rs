use super::CommunityId;
use crate::mock::*;

pub(super) fn get_asset(asset_id: AssetId) -> Option<Vec<u8>> {
	use frame_support::{storage::storage_prefix, Blake2_128Concat, StorageHasher};

	let prefix = storage_prefix(b"Assets", b"Asset");
	let asset_key = Blake2_128Concat::hash(&asset_id.to_le_bytes());

	match sp_io::storage::get(&[prefix.to_vec(), asset_key].concat()) {
		Some(bytes) => Some(bytes.to_vec()),
		None => None,
	}
}

pub(super) fn assert_sufficiency(
	community_id: CommunityId,
	asset_id: AssetId,
	min_balance: Balance,
	is_sufficient: bool,
) {
	use codec::Encode;
	use pallet_assets::AssetDetails;
	use sp_core::hexdisplay::AsBytesRef;

	let community_account_id = Communities::get_community_account_id(&community_id);

	let value = get_asset(asset_id.clone()).expect("we just saved this asset");
	let value_expected = [
		community_account_id.to_le_bytes().to_vec(), // owner
		community_account_id.to_le_bytes().to_vec(), // issuer
		community_account_id.to_le_bytes().to_vec(), // admin
		community_account_id.to_le_bytes().to_vec(), // freezer
		Balance::default().to_le_bytes().to_vec(),   // supply
		Balance::default().to_le_bytes().to_vec(),   // deposit
		min_balance.encode(),                        // min_balance
		is_sufficient.encode(),                      // is_sufficient
		u32::default().encode(),                     // accounts
		u32::default().encode(),                     // sufficients
		u32::default().encode(),                     // approvals
		0u8.encode(),                                // status
	]
	.concat();

	let decoded: AssetDetails<Balance, AccountId, Balance> = codec::Decode::decode(&mut value.as_bytes_ref()).unwrap();
	let expected: AssetDetails<Balance, AccountId, Balance> =
		codec::Decode::decode(&mut value_expected.as_bytes_ref()).unwrap();

	assert_eq!(decoded, expected);
}
