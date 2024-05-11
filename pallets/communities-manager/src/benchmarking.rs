//! Benchmarking setup for pallet-communities
use super::*;

use frame_benchmarking::v2::*;

use frame_support::traits::fungible::Mutate;
use frame_system::RawOrigin;
use sp_runtime::SaturatedConversion;

type RuntimeEventFor<T> = <T as Config>::RuntimeEvent;

fn assert_has_event<T: Config>(generic_event: RuntimeEventFor<T>) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

fn setup_account<T: Config>(who: &AccountIdOf<T>) -> Result<(), BenchmarkError>
where
	NativeBalanceOf<T>: From<u128>,
{
	let initial_balance: NativeBalanceOf<T> = 1_000_000_000_000_000u128.into();
	T::Balances::mint_into(who, initial_balance)?;
	Ok(())
}

#[benchmarks(
where
	RuntimeEventFor<T>: From<pallet_communities::Event<T>>,
	NativeBalanceOf<T>: From<u128>,
	BlockNumberFor<T>: From<u32>,
	CommunityIdOf<T>: From<u16>,
	<T as Config>::MembershipId: From<u32>,
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn register() -> Result<(), BenchmarkError> {
		// setup code
		let first_member: AccountIdOf<T> = frame_benchmarking::account("founder", 0, 0);
		setup_account::<T>(&first_member)?;

		let community_id: CommunityIdOf<T> = 1.into();
		let first_admin = T::Lookup::unlookup(first_member.clone());

		#[extrinsic_call]
		_(
			RawOrigin::Root,
			community_id,
			BoundedVec::truncate_from(b"Test Community".into()),
			first_admin,
			None,
			None,
		);

		// verification code
		assert_has_event::<T>(Event::<T>::CommunityRegistered { id: community_id }.into());
		Ok(())
	}

	#[benchmark]
	fn create_memberships(q: Linear<1, 1024>) -> Result<(), BenchmarkError> {
		// setup code
		T::CreateCollection::create_collection_with_id(
			T::MembershipsManagerCollectionId::get(),
			&T::MembershipsManagerOwner::get(),
			&T::MembershipsManagerOwner::get(),
			&pallet_nfts::CollectionConfig {
				settings: Default::default(),
				max_supply: None,
				mint_settings: Default::default(),
			},
		)?;

		#[extrinsic_call]
		_(
			RawOrigin::Root,
			q.saturated_into(),
			100u32.into(),
			300_000_000_000u128.into(),
		);

		// verification code
		assert_has_event::<T>(
			Event::<T>::MembershipsCreated {
				starting_at: 100u32.into(),
				amount: q.saturated_into(),
			}
			.into(),
		);
		Ok(())
	}

	impl_benchmark_test_suite!(
		Pallet,
		sp_io::TestExternalities::new(Default::default()),
		crate::mock::Test
	);
}
