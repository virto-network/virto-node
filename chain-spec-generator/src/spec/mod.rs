use sc_chain_spec::{ChainSpec, ChainSpecExtension, ChainSpecGroup, ChainType};
use serde::{Deserialize, Serialize};

#[cfg(feature = "paseo")]
const KREIVO_PARA_ID: u32 = 2281;
#[cfg(not(feature = "paseo"))]
const KREIVO_PARA_ID: u32 = 2281;

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	#[serde(alias = "relayChain", alias = "RelayChain")]
	pub relay_chain: String,
	/// The id of the Parachain.
	#[serde(alias = "paraId", alias = "ParaId")]
	pub para_id: u32,
}

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type KreivoChainSpec = sc_chain_spec::GenericChainSpec<Extensions>;

pub mod live;
pub mod local;
