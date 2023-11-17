use frame_support::Parameter;
use sp_runtime::traits::{CheckedAdd, CheckedSub, One};

pub trait Inspect {
	/// Retrieves the current rank of the member
	fn rank(&self) -> Self;
}

pub trait Mutate: Sized {
	/// Retrieves a new rank for the member as increasing by one
	fn promote(&self) -> Option<Self>;
	/// Retrieves a new rank for the member as decreasing by one
	fn demote(&self) -> Option<Self>;
}

impl<Rank> Inspect for Rank
where
	Rank: Default + Parameter + PartialEq + PartialOrd,
{
	fn rank(&self) -> Rank {
		Default::default()
	}
}

impl<Rank> Mutate for Rank
where
	Rank: One + Parameter + PartialEq + PartialOrd + CheckedAdd + CheckedSub,
{
	fn promote(&self) -> Option<Self> {
		self.clone().checked_add(&One::one())
	}

	fn demote(&self) -> Option<Self> {
		self.clone().checked_sub(&One::one())
	}
}
