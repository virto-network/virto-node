#![allow(unused_qualifications)]
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

// TODO : Create communityId as speced in https://github.com/virto-network/virto-node/issues/133
pub type CommunityIdLower = u8;
pub type CommunityIdUpper = u8;
pub type CommunityIdRes = u16;

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommunityId {
    pub lower : CommunityIdLower,
    pub upper : CommunityIdUpper,
    pub res : CommunityIdRes
}

pub type HomeServerUrl = Vec<u8>;