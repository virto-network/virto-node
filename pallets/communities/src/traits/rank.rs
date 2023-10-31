use frame_support::Parameter;

pub trait MemberRank<Rank>
where
	Rank: Default + Parameter + PartialEq + PartialOrd,
{
	fn rank(&self) -> Rank;
}

impl<Rank> MemberRank<Rank> for Rank
where
	Rank: Default + Parameter + PartialEq + PartialOrd,
{
	fn rank(&self) -> Rank {
		Default::default()
	}
}
