#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![cfg_attr(nightly, feature(ascii_char))]

#[cfg(feature = "alloc")]
extern crate alloc;

mod payment_id;
pub use payment_id::PaymentId;
