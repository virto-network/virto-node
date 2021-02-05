use sp_runtime::{
    app_crypto::{app_crypto, sr25519},
    KeyTypeId,
};

pub const OFFCHAIN_KEY_TYPE: KeyTypeId = KeyTypeId(*b"ofcs");

app_crypto!(sr25519, OFFCHAIN_KEY_TYPE);
