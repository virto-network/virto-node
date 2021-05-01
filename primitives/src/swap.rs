#![allow(unused_qualifications)]
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::prelude::*;

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PairPrice<P, R> {
    pub pair: P,
    pub price: R
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Swap<H, P, B> {
    pub human: H,
    pub kind: SwapKind,
    pub price: P,
    pub amount: B,
}
  
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SwapKind {
    In(SwapIn),
    Out(SwapOut),
}

pub type Reason = Vec<u8>;
pub type Proof = Vec<u8>;
pub type Cid = Vec<u8>;

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SwapIn {
    Created,
    Accepted(Cid),
    Rejected(Reason),
    Confirmed(Proof),
    Expired,
    Completed,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SwapOut {
    Created(Cid),
    Accepted,
    Rejected(Reason),
    Confirmed(Proof),
    Expired,
    Completed,
}
