use super::*;

use crate::Communities;
use sp_runtime::SaturatedConversion;
use xcm::v3::{BodyId, Junction::Plurality};
use xcm_executor::traits::ConvertLocation;

pub struct PluralityConvertsToCommunityAccountId;
impl ConvertLocation<AccountId> for PluralityConvertsToCommunityAccountId {
	fn convert_location(location: &MultiLocation) -> Option<AccountId> {
		log::trace!("Attempting to convert {:?} into AccountId if plurality", location);
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
