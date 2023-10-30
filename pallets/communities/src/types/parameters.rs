use super::*;
pub use frame_support::traits::OriginTrait;

pub type AssetIdOf<T> = <<T as Config>::Assets as InspectFuns<AccountIdOf<T>>>::AssetId;
pub type BalanceOf<T> = <<T as Config>::Assets as InspectFuns<AccountIdOf<T>>>::Balance;
pub type NativeBalanceOf<T> = <<T as Config>::Balances as Inspect<AccountIdOf<T>>>::Balance;
pub type AccountIdOf<T> = <T as SystemConfig>::AccountId;
pub type AccountIdLookupOf<T> = <<T as SystemConfig>::Lookup as StaticLookup>::Source;

pub type CommunityIdOf<T> = <T as Config>::CommunityId;
pub type MemberListOf<T> = Vec<AccountIdOf<T>>;

pub type PalletsOriginOf<T> = <T as Config>::PalletsOrigin;
pub type RuntimeCallOf<T> = <T as SystemConfig>::RuntimeCall;

pub type MembershipPassportOf<T> = <T as Config>::MembershipPassport;
pub type MembershipRankOf<T> = <T as Config>::MembershipRank;
pub type VoteWeightFor<T> = <T as Config>::VoteWeight;

pub type Cell = u32;
