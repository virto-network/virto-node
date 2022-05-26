use super::*;
use crate::Pallet as Communities;
use crate::{Communities as CommunitiesStore, CommunityId, DomainNameOf};
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::{EnsureOrigin, Get};
use frame_system::RawOrigin;
use sp_std::prelude::*;

const SEED: u32 = 0;

benchmarks! {
	register {
		let caller = whitelisted_caller();
		let domain : DomainNameOf = vec![1;8].try_into().unwrap();
		let id = CommunityId {
			base : 1,
			category : 1,
			instance : 1
		};
	}: _(RawOrigin::Signed(caller.clone()), id, domain)
	verify {}

	impl_benchmark_test_suite!(Communities, crate::mock::new_test_ext(), crate::mock::Test);
}
