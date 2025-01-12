#![cfg_attr(not(any(test, feature = "std")), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod multilocation_asset_id;
mod payment_id;

pub use multilocation_asset_id::{FungibleAssetLocation, NetworkId};
pub use payment_id::PaymentId;

#[cfg(feature = "runtime")]
pub use multilocation_asset_id::runtime::AsFungibleAssetLocation;

pub type CommunityId = u16;
pub type MembershipId = u32;
