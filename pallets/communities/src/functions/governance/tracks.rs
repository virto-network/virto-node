use super::*;

use pallet_referenda::{Track, TracksInfo};
use sp_std::borrow::Cow;

// #[cfg(feature = "referenda")]
impl<T: Config> TracksInfo<VoteWeight, BlockNumberFor<T>> for CommunityTracks<T> {
	type Id = T::CommunityId;
	type RuntimeOrigin = RuntimeOriginOf<T>;

	fn tracks() -> impl Iterator<Item = Cow<'static, Track<Self::Id, types::VoteWeight, BlockNumberFor<T>>>> {
		//TODO: Convert from a list of community IDs to list of tracks"

		// Info::<T>::iter_keys().map(|community_id| Track {
		// 	id: community_id,
		// 	info: TrackInfo {
		// 		name: "".into(),
		// 		max_deciding: 0,
		// 		decision_deposit: 0,
		// 		prepare_period: 100,
		// 		decision_period: 100,
		// 		confirm_period: 100,
		// 		min_enactment_period: 100,
		// 		min_approval: pallet_referenda::Curve::LinearDecreasing {
		// 			length: 1.into(),
		// 			floor: 1.into(),
		// 			ceil: 1.into(),
		// 		},
		// 		min_support: pallet_referenda::Curve::LinearDecreasing {
		// 			length: 1.into(),
		// 			floor: 1.into(),
		// 			ceil: 1.into(),
		// 		},
		// 	},
		// })

		vec![].into_iter()
	}

	fn track_for(_origin: &Self::RuntimeOrigin) -> Result<Self::Id, ()> {
		todo!("Convert the signed as community account id into the track Id for the community track")
	}
}
