//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

#[derive(Debug)]
pub struct DefaultWeightInfo;

impl WeightInfo for DefaultWeightInfo {
    fn attest() -> Weight {
        (163_972_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn submit_pair_prices() -> Weight {
        (38_171_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn transfer() -> Weight {
        (74_911_000 as Weight).saturating_add(DbWeight::get().reads(2 as Weight))
    }
    fn update_offer_rates() -> Weight {
        (2_762_000 as Weight)
    }
}

pub trait WeightInfo {
    fn attest() -> Weight;
    fn submit_pair_prices() -> Weight;
    fn transfer() -> Weight;
    fn update_offer_rates() -> Weight;
}
