use super::*;

use crate::Communities;
use sp_runtime::SaturatedConversion;
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
