use super::*;

use pallet_referenda::{impl_tracksinfo_get, Track};
use sp_runtime::str_array as s;
use sp_std::borrow::Cow;

pub type TrackId = u16;

pub struct TracksInfo;
impl pallet_referenda::TracksInfo<Balance, BlockNumber> for TracksInfo {
	type Id = TrackId;
	type RuntimeOrigin = <RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin;
	type TracksIter = pallet_referenda::StaticTracksIter<Self::Id, Balance, BlockNumber>;

	fn tracks() -> Self::TracksIter {
		const DATA: [pallet_referenda::Track<TrackId, Balance, BlockNumber>; 1] = [Track {
			id: 0,
			info: pallet_referenda::TrackInfo {
				name: s("Root"),
				max_deciding: 1,
				decision_deposit: 0,
				prepare_period: 15 * MINUTES,
				decision_period: 4 * DAYS,
				confirm_period: 15 * MINUTES,
				min_enactment_period: 1,
				min_approval: pallet_referenda::Curve::LinearDecreasing {
					length: Perbill::from_percent(100),
					floor: Perbill::from_percent(90),
					ceil: Perbill::from_percent(100),
				},
				min_support: pallet_referenda::Curve::LinearDecreasing {
					length: Perbill::from_percent(100),
					floor: Perbill::from_percent(0),
					ceil: Perbill::from_percent(100),
				},
			},
		}];
		DATA.iter().map(Cow::Borrowed)
	}

	fn track_for(id: &Self::RuntimeOrigin) -> Result<Self::Id, ()> {
		if let Ok(system_origin) = frame_system::RawOrigin::try_from(id.clone()) {
			match system_origin {
				frame_system::RawOrigin::Root => Ok(0),
				_ => Err(()),
			}
		} else {
			Err(())
		}
	}
}
impl_tracksinfo_get!(TracksInfo, Balance, BlockNumber);
