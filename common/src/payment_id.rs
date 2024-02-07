#[cfg(feature = "js")]
use wasm_bindgen::prelude::*;

/// A compact identifier for payment
#[cfg_attr(feature = "js", wasm_bindgen)]
#[derive(Debug, Default, Clone, Eq, Copy, PartialEq)]
#[cfg_attr(
	feature = "runtime",
	derive(
		parity_scale_codec::Encode,
		parity_scale_codec::Decode,
		parity_scale_codec::MaxEncodedLen,
		scale_info::TypeInfo,
	)
)]
#[repr(C)]
pub struct PaymentId {
	prefix: [u8; 2],
	index: u16,
	block: u32,
}

#[cfg_attr(feature = "js", wasm_bindgen)]
impl PaymentId {
	#[cfg_attr(feature = "js", wasm_bindgen(constructor))]
	#[cfg(feature = "nightly")]
	pub fn new(id: &str) -> PaymentId {
		id.parse().unwrap_or(Default::default())
	}

	#[cfg_attr(feature = "js", wasm_bindgen(js_name = fromNumber))]
	pub fn from_number(n: u64) -> PaymentId {
		n.into()
	}

	#[cfg_attr(feature = "js", wasm_bindgen(getter = blockNumber))]
	pub fn block_number(&self) -> u32 {
		self.block
	}

	#[cfg_attr(feature = "js", wasm_bindgen(getter = extrinsicIndex))]
	pub fn extrinsic_index(&self) -> u32 {
		self.index as u32
	}

	#[cfg(all(feature = "nightly", feature = "alloc"))]
	pub fn encode(&self, pretty: bool) -> alloc::string::String {
		if pretty {
			alloc::format!("{self:#}")
		} else {
			alloc::format!("{self}")
		}
	}

	#[cfg_attr(feature = "js", wasm_bindgen(js_name = toNumber))]
	pub fn to_number(&self) -> u64 {
		(*self).into()
	}

	#[cfg(feature = "alloc")]
	pub fn to_bytes(&self) -> alloc::vec::Vec<u8> {
		self.as_ref().into()
	}
}

impl From<PaymentId> for u64 {
	fn from(id: PaymentId) -> Self {
		u64::from_le_bytes(id.as_ref().try_into().expect("fits in u64"))
	}
}

impl From<u64> for PaymentId {
	fn from(value: u64) -> Self {
		let val = value.to_le_bytes();
		let index = u16::from_le_bytes(val[2..4].try_into().unwrap());
		let block = u32::from_le_bytes(val[4..].try_into().unwrap());
		PaymentId {
			prefix: [val[0], val[1]],
			block,
			index,
		}
	}
}

impl From<(u32, u32, &[u8])> for PaymentId {
	fn from((block, idx, extra): (u32, u32, &[u8])) -> Self {
		debug_assert!(extra.len() >= 2);
		PaymentId {
			prefix: extra[..2].try_into().expect("at least 2 bytes"),
			block,
			index: idx as u16,
		}
	}
}

impl AsRef<[u8]> for PaymentId {
	fn as_ref(&self) -> &[u8] {
		debug_assert_eq!(8, core::mem::size_of::<Self>());
		unsafe { core::slice::from_raw_parts((self as *const PaymentId) as *const u8, core::mem::size_of::<Self>()) }
	}
}

#[cfg(feature = "nightly")]
impl core::str::FromStr for PaymentId {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut tmp = [0u8; 12];
		let mut input = s;
		if let Some((p1, p2)) = s.split_once('-') {
			let len_without_sep = p1.len() + p2.len();
			debug_assert!(len_without_sep <= 12);
			tmp[..p1.len()].copy_from_slice(p1.as_bytes());
			tmp[p1.len()..len_without_sep].copy_from_slice(p2.as_bytes());
			input = tmp[..len_without_sep].as_ascii().ok_or(())?.as_str();
		};
		let mut out = [0u8; 8];
		let _ = bs58::decode(input).onto(&mut out[..]).map_err(|_| ())?;
		Ok(u64::from_le_bytes(out).into())
	}
}

#[cfg(feature = "nightly")]
impl core::fmt::Display for PaymentId {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		use core::fmt::Error;
		let mut out = [0u8; 12];
		let n = bs58::encode(self).onto(&mut out[..]).map_err(|_| Error)?;
		let out = out[..n].as_ascii().ok_or(Error)?.as_str();
		write!(f, "{}", &out[..5])?;
		if f.alternate() {
			write!(f, "-")?;
		}
		write!(f, "{}", &out[5..])
	}
}

#[cfg(all(test, feature = "nightly"))]
mod tests {
	extern crate alloc;
	use super::*;
	use alloc::format;

	#[test]
	fn payment_id_u64() {
		let id: PaymentId = u64::MAX.into();
		assert_eq!(id, (u32::MAX, u32::MAX, &[0xFF, 0xFF][..]).into());
	}

	#[test]
	fn payment_id_display() {
		let id: PaymentId = u64::MAX.into();
		assert_eq!(format!("{id}"), "jpXCZedGfVQ");
		assert_eq!(format!("{id:#}"), "jpXCZ-edGfVQ");

		const TEST_ID: &str = "LbNvS-NtVQs";
		let id: PaymentId = (1_234_567, 5, &[2, 5][..]).into();
		assert_eq!(format!("{id:#}"), TEST_ID);
		assert_eq!(id, TEST_ID.parse().unwrap());
	}
}
