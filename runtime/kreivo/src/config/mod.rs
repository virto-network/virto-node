//! Configure FRAME pallets to include in runtime.

use super::*;

mod collator_support;
pub mod currency;
pub mod system;
mod utilities;
mod xcm;
// Kreivo Governance
pub mod collective;
pub mod communities;
pub mod governance;
// Virto toolchain
pub mod contracts;
pub mod payments;

pub use collator_support::{ConsensusHook, SLOT_DURATION};
pub use currency::{
	ExistentialDeposit, KreivoAssetsCall, KreivoAssetsInstance, MembershipsGasTank, MetadataDepositBase,
	MetadataDepositPerByte, TransactionByteFee,
};
pub use governance::{pallet_custom_origins, TreasuryAccount};
pub use system::RuntimeBlockWeights;
pub use xcm::PriceForParentDelivery;
