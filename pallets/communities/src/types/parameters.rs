use super::*;
pub use frame_support::traits::{fungibles, OriginTrait};

pub type AssetIdOf<T> = <<T as Config>::Assets as fungibles::Inspect<AccountIdOf<T>>>::AssetId;
pub type NativeBalanceOf<T> = <<T as Config>::Balances as Inspect<AccountIdOf<T>>>::Balance;
pub type AccountIdOf<T> = <T as SystemConfig>::AccountId;
pub type AccountIdLookupOf<T> = <<T as SystemConfig>::Lookup as StaticLookup>::Source;

pub type CommunityIdOf<T> = <T as Config>::CommunityId;
pub type MemberListOf<T> = Vec<AccountIdOf<T>>;

pub type PalletsOriginOf<T> = <T as Config>::PalletsOrigin;
pub type RuntimeCallOf<T> = <T as SystemConfig>::RuntimeCall;

pub type MembershipOf<T> = <T as Config>::Membership;
