use super::*;

pub type AssetIdOf<T> = <<T as Config>::Assets as InspectFuns<AccountIdOf<T>>>::AssetId;
pub type BalanceOf<T> = <<T as Config>::Assets as InspectFuns<AccountIdOf<T>>>::Balance;
pub type NativeBalanceOf<T> = <<T as Config>::Balances as Inspect<AccountIdOf<T>>>::Balance;
pub type AccountIdOf<T> = <T as SystemConfig>::AccountId;
pub type AccountIdLookupOf<T> = <<T as SystemConfig>::Lookup as StaticLookup>::Source;
pub type CommunityIdOf<T> = <T as Config>::CommunityId;
pub type MembershipPassportOf<T> = <T as Config>::MembershipPassport;
pub type MemberListOf<T> = Vec<AccountIdOf<T>>;
pub type RuntimeCallOf<T> = <T as SystemConfig>::RuntimeCall;

pub type Cell = u32;
