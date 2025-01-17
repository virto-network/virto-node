//! Configure FRAME pallets to include in runtime.

use super::*;

pub mod monetary_stuff;
pub mod system_support;

pub use monetary_stuff::{
	ExistentialDeposit, KreivoAssetsCall, KreivoAssetsInstance, MembershipsGasTank, MetadataDepositBase,
	MetadataDepositPerByte, TransactionByteFee,
};
pub use system_support::RuntimeBlockWeights;
