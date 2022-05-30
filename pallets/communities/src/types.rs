use crate::{Config, DomainNameOf};
use frame_support::RuntimeDebug;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::traits::Zero;
use sp_std::{fmt, num::ParseIntError, str::FromStr, vec::Vec};

#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound(T: Config))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Community<T: Config> {
	// TODO : Maybe every community can configure an asset?
	pub controller: T::AccountId,
	pub population: u32,
	pub domain_name: DomainNameOf<T>,
}

pub type BaseIndex = u8;
pub type CategoryIndex = u8;
pub type InstanceIndex = u16;

/// CommunityId representation
#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, Default, TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommunityId {
	pub base: BaseIndex,
	pub category: CategoryIndex,
	pub instance: InstanceIndex,
}

impl CommunityId {
	pub fn is_valid(&self) -> bool {
		// zero base index is reserved
		if self.base.is_zero() {
			return false;
		}

		true
	}
}

impl fmt::Display for CommunityId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "({}.{}.{})", self.base, self.category, self.instance)
	}
}

impl FromStr for CommunityId {
	type Err = ParseIntError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let coords: Vec<&str> = s.trim_matches(|p| p == '(' || p == ')').split('.').collect();

		let base_fromstr = coords[0].parse::<u8>()?;
		let category_fromstr = coords[1].parse::<u8>()?;
		let instance_fromstr = coords[2].parse::<u16>()?;

		Ok(CommunityId {
			base: base_fromstr,
			category: category_fromstr,
			instance: instance_fromstr,
		})
	}
}
