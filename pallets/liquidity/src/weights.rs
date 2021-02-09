//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_imports, unused_parens)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

#[derive(Debug)]
pub struct DefaultWeightInfo;

impl WeightInfo for DefaultWeightInfo {
    fn attest() -> Weight {
        (163_972_000_u64)
            .saturating_add(DbWeight::get().reads(5_u64))
            .saturating_add(DbWeight::get().writes(4_u64))
    }
    fn members() -> Weight {
        (34_129_000_u64).saturating_add(DbWeight::get().reads(1_u64))
    }
    fn transfer() -> Weight {
        (74_911_000_u64).saturating_add(DbWeight::get().reads(2_u64))
    }
    fn update_offer_rates() -> Weight {
        (2_762_000_u64)
    }
}

pub trait WeightInfo {
    fn attest() -> Weight;
    fn members() -> Weight;
    fn transfer() -> Weight;
    fn update_offer_rates() -> Weight;
}
