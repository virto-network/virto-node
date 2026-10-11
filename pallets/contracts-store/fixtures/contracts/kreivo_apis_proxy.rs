//! Forwards its input to the chain extension, or to another contract.
//!
//! - `0x00 ++ func_id: u32 ++ input`: calls the chain extension function
//!   `func_id` with `input`, and returns `ret_code: u32 ++ output`.
//! - `0x01 ++ callee: [u8; 32] ++ input`: calls `callee` with `input`, and
//!   returns its output.
#![no_std]
#![no_main]

extern crate common;

use uapi::{CallFlags, HostFn, HostFnImpl as api, ReturnFlags};

#[no_mangle]
#[polkavm_derive::polkavm_export]
pub extern "C" fn deploy() {}

#[no_mangle]
#[polkavm_derive::polkavm_export]
pub extern "C" fn call() {
	// The buffers are not zeroed: zeroing them emits bulk-memory instructions,
	// which the fixtures' post-processing can't parse. Only the bytes the host
	// writes are ever read.
	let mut input = core::mem::MaybeUninit::<[u8; 256]>::uninit();
	let mut output = core::mem::MaybeUninit::<[u8; 256]>::uninit();
	// SAFETY: `u8` has no invalid bit patterns, and only written bytes are read.
	let (input, output) = unsafe { (&mut input.assume_init_mut()[..], &mut output.assume_init_mut()[..]) };
	let input = &mut &mut input[..];

	api::input(input);
	let (mode, rest) = (input[0], &input[1..]);

	match mode {
		0 => {
			let func_id = u32::from_le_bytes([rest[0], rest[1], rest[2], rest[3]]);
			let (code, out) = output.split_at_mut(4);
			let mut out = out;
			let ret_code = api::call_chain_extension(func_id, &rest[4..], Some(&mut out));
			let len = out.len();
			let ret_code = ret_code.to_le_bytes();
			code[0] = ret_code[0];
			code[1] = ret_code[1];
			code[2] = ret_code[2];
			code[3] = ret_code[3];
			api::return_value(ReturnFlags::empty(), &output[..4 + len]);
		}
		_ => {
			let mut out = &mut output[..];
			api::call_v2(
				CallFlags::empty(),
				&rest[..32],
				0u64,                 // All the remaining ref_time.
				0u64,                 // All the remaining proof_size.
				None,                 // No deposit limit.
				&0u128.to_le_bytes(), // No value transferred.
				&rest[32..],
				Some(&mut out),
			)
			.unwrap();
			let len = out.len();
			api::return_value(ReturnFlags::empty(), &output[..len]);
		}
	}
}
