use super::*;
use crate::Pallet as Communities;
use crate::{Communities as CommunitiesStore, CommunityId, DomainNameOf};
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use sp_std::prelude::*;

benchmarks! {
	register {
		let caller : T::AccountId = whitelisted_caller();
		let domain : DomainNameOf<T> = vec![1;8].try_into().unwrap();
		let id = CommunityId {
			base : 1,
			category : 1,
			instance : 1
		};
	}: _(RawOrigin::Signed(caller.clone()), id, domain.clone())
	verify {
		assert_eq!(CommunitiesStore::<T>::get((1,1,1)).unwrap().domain_name, domain);
	}

	impl_benchmark_test_suite!(Communities, crate::mock::new_test_ext(), crate::mock::Test);
}
