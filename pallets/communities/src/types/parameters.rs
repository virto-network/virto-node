use super::*;
use frame_support::traits::Polling;
pub use frame_support::traits::{fungibles, OriginTrait};

pub type AssetIdOf<T> = <<T as Config>::Assets as fungibles::Inspect<AccountIdOf<T>>>::AssetId;
pub type AssetBalanceOf<T> = <<T as Config>::Assets as fungibles::Inspect<AccountIdOf<T>>>::Balance;
pub type NativeBalanceOf<T> = <<T as Config>::Balances as Inspect<AccountIdOf<T>>>::Balance;
pub type AccountIdOf<T> = <T as SystemConfig>::AccountId;
pub type AccountIdLookupOf<T> = <<T as SystemConfig>::Lookup as StaticLookup>::Source;
pub type CommunityIdOf<T> = <T as Config>::CommunityId;
pub type MemberListOf<T> = Vec<AccountIdOf<T>>;
pub type MembershipOf<T> = <T as Config>::Membership;
pub type VoteOf<T> = Vote<AssetIdOf<T>, AssetBalanceOf<T>>;
pub type PollIndexOf<T> = <<T as Config>::Polls as Polling<Tally<T>>>::Index;
pub type RuntimeOriginOf<T> = <<T as SystemConfig>::RuntimeOrigin as OriginTrait>::PalletsOrigin;
