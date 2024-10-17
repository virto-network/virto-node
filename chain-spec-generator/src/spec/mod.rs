use crate::common::{get_account_id_from_seed, get_collator_keys_from_seed, Extensions, SAFE_XCM_VERSION};
use cumulus_primitives_core::ParaId;

use kreivo_runtime::{constants::currency::EXISTENTIAL_DEPOSIT, AccountId, AuraId, SessionKeys};
use sc_chain_spec::{ChainSpec, ChainType};
use sp_core::sr25519;

#[cfg(feature = "paseo")]
const KREIVO_PARA_ID: u32 = 2281;
#[cfg(not(feature = "paseo"))]
const KREIVO_PARA_ID: u32 = 2281;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type KreivoChainSpec = sc_chain_spec::GenericChainSpec<Extensions>;

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we
/// have just one key).
fn session_keys(aura: AuraId) -> SessionKeys {
	SessionKeys { aura }
}

pub mod live;
pub mod local;
