use crate::{mock::*, Communities as CommunitiesStorage, Community, CommunityId};
use frame_support::{assert_noop, assert_ok, storage::with_transaction};
use orml_traits::{MultiCurrency, MultiReservableCurrency, NamedMultiReservableCurrency};
use sp_runtime::{Percent, TransactionOutcome};

type Error = crate::Error<Test>;

fn last_event() -> Event {
	System::events().pop().expect("Event expected").event
}

#[test]
fn test_register_works() {
	new_test_ext().execute_with(|| {
		let controller = 1;
		let community_id = CommunityId {
			index: 100,
			res: 100,
			value: 100,
		};

		// should be able to register a new community
		assert_ok!(Communities::register(
			Origin::signed(controller),
			community_id.clone(),
			vec![1u8; 10].try_into().unwrap()
		));

		assert_eq!(
			last_event(),
			crate::Event::<Test>::CommunityRegistered(community_id.clone()).into()
		);

		// should be stored correctly
		assert_eq!(
			CommunitiesStorage::<Test>::get((community_id.index, community_id.res, community_id.value)),
			Some(Community {
				controller,
				population: Default::default(),
				domain_name: vec![1u8; 10].try_into().unwrap()
			})
		);
	});
}
