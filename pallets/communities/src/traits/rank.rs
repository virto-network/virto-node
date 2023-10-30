use frame_support::Parameter;
use sp_runtime::traits::AtLeast32BitUnsigned;

pub trait MemberRank<Rank>
where
	Rank: Default + AtLeast32BitUnsigned + Parameter + PartialEq + PartialOrd,
{
	fn rank(&self) -> Rank;
}

impl<Rank> MemberRank<Rank> for Rank
where
	Rank: Default + AtLeast32BitUnsigned + Parameter + PartialEq + PartialOrd,
{
	fn rank(&self) -> Rank {
		Default::default()
	}
}
