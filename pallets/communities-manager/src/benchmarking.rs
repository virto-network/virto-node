//! Benchmarking setup for pallet-communities
use super::*;

use frame_benchmarking::v2::*;

use frame_support::{dispatch::DispatchResult, traits::fungible::Mutate};
use frame_system::RawOrigin;
use pallet_communities::BenchmarkHelper;
use sp_runtime::traits::Saturating;

fn assert_has_event<T: frame_system::Config>(generic_event: T::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_has_event(generic_event);
}

fn setup_account<T: Config>(who: &AccountIdOf<T>) -> DispatchResult
where
	NativeBalanceOf<T>: From<u64>,
{
	let initial_balance: NativeBalanceOf<T> = 1_000_000_000_000_000u64.into();
	T::Balances::mint_into(who, initial_balance)?;
	Ok(())
}

/// A community, registered for free, whose account can hold the deposits of `memberships` more
/// memberships. The community is the one the group origin names.
fn community_with_funds<T: Config>(memberships: u32) -> Result<CommunityIdOf<T>, BenchmarkError>
where
	NativeBalanceOf<T>: From<u64>,
{
	// The group origin only recognises a community that exists, so the community is registered
	// before the origin is checked against it.
	let community = <T as pallet_communities::Config>::BenchmarkHelper::community_id();

	let admin: AccountIdOf<T> = account("admin", 0, 0);
	Pallet::<T>::try_register(None, community, "Benchmark", &admin, None, None, None)?;

	let origin = T::GroupAdminOrigin::try_successful_origin()
		.map_err(|_| BenchmarkError::Stop("the runtime has no successful group origin"))?;
	if T::GroupAdminOrigin::try_origin(origin).ok() != Some(community) {
		return Err(BenchmarkError::Stop(
			"the group origin does not name the benchmark community",
		));
	}

	let community_account = pallet_communities::Pallet::<T>::community_account(&community);
	let funds = T::MembershipDeposit::get().saturating_mul((memberships as u64 + 1).into());
	T::Balances::mint_into(
		&community_account,
		funds.saturating_add(1_000_000_000_000_000u64.into()),
	)?;
	Ok(community)
}

#[cfg(feature = "std")]
pub use benchmarks::test_group_benchmark;

#[benchmarks(
where
	T::RuntimeEvent: From<pallet_communities::Event<T>>,
	NativeBalanceOf<T>: From<u64>,
	CommunityIdOf<T>: One,
	T::MembershipId: From<u32>,
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn register() -> Result<(), BenchmarkError> {
		// setup code
		let first_member: AccountIdOf<T> = account("founder", 0, 0);
		setup_account::<T>(&first_member)?;

		// The worst case is a registration someone pays for, if the runtime lets anyone register.
		let signed: OriginFor<T> = RawOrigin::Signed(first_member.clone()).into();
		let origin = if T::RegisterOrigin::try_origin(signed.clone()).is_ok() {
			signed
		} else {
			RawOrigin::Root.into()
		};

		let community_id: CommunityIdOf<T> = One::one();

		#[extrinsic_call]
		_(
			origin as T::RuntimeOrigin,
			community_id,
			BoundedVec::truncate_from(b"Test Community".into()),
			T::Lookup::unlookup(first_member.clone()),
			None,
			None,
		);

		// verification code
		assert_has_event::<T>(Event::<T>::CommunityRegistered { id: community_id }.into());
		Ok(())
	}

	#[benchmark]
	fn issue_memberships(n: Linear<1, { T::MaxIssuePerCall::get() }>) -> Result<(), BenchmarkError> {
		let community = community_with_funds::<T>(n)?;
		let origin = T::GroupAdminOrigin::try_successful_origin()
			.map_err(|_| BenchmarkError::Stop("the runtime has no successful group origin"))?;

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, n);

		// verification code
		assert_eq!(Deposits::<T>::iter().count() as u32, n);
		assert!(Deposits::<T>::contains_key((community, T::MembershipId::from(0u32))));
		Ok(())
	}

	#[benchmark]
	fn retire_memberships(n: Linear<1, { T::MaxIssuePerCall::get() }>) -> Result<(), BenchmarkError> {
		let community = community_with_funds::<T>(n)?;
		let origin = T::GroupAdminOrigin::try_successful_origin()
			.map_err(|_| BenchmarkError::Stop("the runtime has no successful group origin"))?;

		let mut id = NextMembershipId::<T>::get();
		let mut ids = Vec::new();
		Pallet::<T>::do_issue_memberships(&community, n)?;
		for _ in 0..n {
			ids.push(id);
			id = id.increment().ok_or(BenchmarkError::Stop("membership ids ran out"))?;
		}
		let ids = BoundedVec::try_from(ids).map_err(|_| BenchmarkError::Stop("too many ids for one call"))?;

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, ids);

		// verification code
		assert_eq!(Retiring::<T>::iter().count() as u32, n);
		Ok(())
	}

	#[benchmark]
	fn complete_retirement(a: Linear<1, 64>) -> Result<(), BenchmarkError> {
		let community = community_with_funds::<T>(1)?;
		let id = NextMembershipId::<T>::get();
		Pallet::<T>::do_issue_memberships(&community, 1)?;
		// An issued membership already carries the attribute the issuer wrote, so the item carries
		// `a` attributes in all.
		let written = T::ItemAttributes::count(&community, &id);
		T::ItemAttributes::set_benchmark_attributes(&community, &id, a.saturating_sub(written));
		Pallet::<T>::do_retire_memberships(&community, &[id])?;
		// The delay has passed.
		Retiring::<T>::insert((community, id), MomentOf::<T>::zero());

		let caller: AccountIdOf<T> = account("caller", 0, 0);

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), community, id, a);

		// verification code
		assert!(Retiring::<T>::get((community, id)).is_none());
		assert!(Deposits::<T>::get((community, id)).is_none());
		Ok(())
	}

	/// Runs the setup, the call and the verification of one of the group benchmarks, as a runtime's
	/// tests do, in a fresh state. A skipped (weightless) benchmark would pass unnoticed, so the
	/// setups stop on errors instead.
	#[cfg(feature = "std")]
	pub fn test_group_benchmark<T: Config>(name: &str) -> Result<(), BenchmarkError>
	where
		T::RuntimeEvent: From<pallet_communities::Event<T>>,
		NativeBalanceOf<T>: From<u64>,
		CommunityIdOf<T>: One,
		T::MembershipId: From<u32>,
	{
		use frame_benchmarking::{BenchmarkParameter, BenchmarkingSetup};

		let (benchmark, component) = match name {
			"issue_memberships" => (SelectedBenchmark::issue_memberships, BenchmarkParameter::n),
			"retire_memberships" => (SelectedBenchmark::retire_memberships, BenchmarkParameter::n),
			"complete_retirement" => (SelectedBenchmark::complete_retirement, BenchmarkParameter::a),
			_ => return Err(BenchmarkError::Stop("not a group benchmark")),
		};
		let before_start = || frame_system::Pallet::<T>::set_block_number(1u32.into());
		<SelectedBenchmark as BenchmarkingSetup<T, _>>::test_instance(&benchmark, &[(component, 4)], &before_start)
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), mock::Test);
}
