#![no_std]
#![feature(ascii_char)]
#[cfg(feature = "alloc")]
extern crate alloc;

mod payment_id;
pub use payment_id::PaymentId;
