use super::*;
pub(self) use frame_support::traits::{schedule::v3::Anon, StorePreimage};
pub(self) use sp_runtime::traits::Zero;

impl<T: Config> Pallet<T> {
	pub(crate) fn do_create_proposal(
		proposer: &AccountIdOf<T>,
		community_id: &CommunityIdOf<T>,
		call_origin: PalletsOriginOf<T>,
		call: RuntimeCallOf<T>,
	) -> DispatchResult {
		Self::ensure_proposal_origin(community_id, call_origin.clone())?;

		let bounded_call = T::Preimage::bound(call).map_err(|_| Error::<T>::CannotEncodeCall)?;

		Self::do_enqueue_proposal(
			community_id,
			CommunityProposal {
				proposer: proposer.clone(),
				call: bounded_call,
				origin: call_origin,
			},
		)?;

		Ok(())
	}

	#[allow(dead_code)]
	pub(crate) fn do_execute_proposal(proposal: CommunityProposal<T>) -> DispatchResult {
		T::Scheduler::schedule(
			frame_support::traits::schedule::DispatchTime::After(Zero::zero()),
			None,
			Default::default(),
			proposal.origin,
			proposal.call,
		)
		.and_then(|_| Ok(()))
	}

	fn do_enqueue_proposal(community_id: &CommunityIdOf<T>, proposal: CommunityProposal<T>) -> DispatchResult {
		if Proposals::<T>::decode_len(community_id).unwrap_or_default() >= T::MaxProposals::get() as usize {
			Err(Error::<T>::ExceededMaxProposals)?;
		}

		Proposals::<T>::try_append(community_id, proposal).map_err(|_| Error::<T>::CannotEnqueueProposal)?;

		Ok(())
	}

	pub(crate) fn do_deequeue_proposal(community_id: &CommunityIdOf<T>) -> Result<CommunityProposal<T>, DispatchError> {
		Proposals::<T>::try_mutate(community_id, |proposals| {
			let first_proposal = proposals.first().ok_or(Error::<T>::CannotDequeueProposal)?.clone();
			proposals.remove(0);

			Ok(first_proposal)
		})
	}
}
