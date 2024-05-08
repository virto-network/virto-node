use super::*;

use crate::Communities;
use core::marker::PhantomData;
use frame_support::{traits::OriginTrait, PalletId};
use pallet_communities::AccountIdOf;
use sp_runtime::{
	traits::{AccountIdConversion, TryConvert},
	SaturatedConversion,
};
use xcm::v3::{BodyId, Junction::Plurality};
use xcm_executor::traits::ConvertLocation;

pub struct PluralityConvertsToCommunityAccountId;
impl ConvertLocation<AccountId> for PluralityConvertsToCommunityAccountId {
	fn convert_location(location: &MultiLocation) -> Option<AccountId> {
		match location {
			MultiLocation {
				parents: 0,
				interior: X1(Plurality {
					id: BodyId::Index(id), ..
				}),
			} => Some(Communities::community_account(&(*id).saturated_into())),
			_ => None,
		}
	}
}

pub struct AccountId32FromRelay<Network, AccountId>(PhantomData<(Network, AccountId)>);
impl<Network: Get<Option<NetworkId>>, AccountId: From<[u8; 32]> + Into<[u8; 32]> + Clone> ConvertLocation<AccountId>
	for AccountId32FromRelay<Network, AccountId>
{
	fn convert_location(location: &MultiLocation) -> Option<AccountId> {
		let id = match location {
			MultiLocation {
				parents: 1,
				interior: X1(AccountId32 { id, network: None }),
			} => id,
			MultiLocation {
				parents: 1,
				interior: X1(AccountId32 { id, network }),
			} if *network == Network::get() => id,
			_ => return None,
		};

		Some((*id).into())
	}
}

pub struct SignedByCommunityToPlurality<T>(PhantomData<T>);
impl<T, OuterOrigin> TryConvert<OuterOrigin, xcm::v3::MultiLocation> for SignedByCommunityToPlurality<T>
where
	OuterOrigin: OriginTrait<AccountId = AccountIdOf<T>> + Clone,
	T: pallet_communities::Config,
	xcm::v3::Junction: TryFrom<pallet_communities::Origin<T>>,
{
	fn try_convert(o: OuterOrigin) -> Result<xcm::v3::MultiLocation, OuterOrigin> {
		let Some(account_id) = o.clone().into_signer() else {
			return Err(o.clone());
		};
		let Some((_, community_id)) = PalletId::try_from_sub_account(&account_id) else {
			return Err(o.clone());
		};
		let origin = pallet_communities::Origin::<T>::new(community_id);
		let j = xcm::v3::Junction::try_from(origin).map_err(|_| o)?;
		Ok(j.into())
	}
}
