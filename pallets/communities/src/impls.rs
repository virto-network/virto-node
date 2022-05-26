use crate::{Communities, CommunityId, Config};
use orml_payments::{FeeHandler, FeeRecipientShareList, PaymentDetail};
use sp_std::str::FromStr;

/// Struct to implement community fee handler
pub struct CommunityFeeHandler;

/// FeeHandler trait implementation for Communities
impl<T: Config> FeeHandler<T> for CommunityFeeHandler {
	fn apply_fees(
		_from: &T::AccountId,
		_to: &T::AccountId,
		_detail: &PaymentDetail<T>,
		remark: Option<&[u8]>,
	) -> FeeRecipientShareList<T> {
		if let Some(remark) = remark {
			let remark_string = sp_std::str::from_utf8(remark).unwrap_or_default();
			let community_id = CommunityId::from_str(remark_string).unwrap_or_default();
			// check if the community exists
			let _community = Communities::<T>::get((community_id.base, community_id.category, community_id.instance));

			// TODO : Apply the community fee config
		}
		Default::default()
	}
}
