use super::*;

type AssetsAPIOf<T, E> = <RuntimeKreivoAPI<T> as KreivoAPI<E>>::Assets;

impl<T, E> ChainExtensionDispatch<E> for AssetsApiInfo<T>
where
	T: Config,
	E: Ext<T = T>,
{
	fn call(&self, ext: &E) -> Result<Vec<u8>, KreivoApisError> {
		match self {
			AssetsApiInfo::Balance { asset, who } => Ok(AssetsAPIOf::<T, E>::balance(ext, asset.clone(), who).encode()),
			AssetsApiInfo::Deposit { asset, amount } => {
				AssetsAPIOf::<T, E>::deposit(ext, asset.clone(), *amount).map(|v| v.encode())
			}
			AssetsApiInfo::Transfer {
				asset,
				amount,
				beneficiary,
			} => AssetsAPIOf::<T, E>::transfer(ext, asset.clone(), *amount, beneficiary).map(|v| v.encode()),
			AssetsApiInfo::Mint {
				asset,
				amount,
				beneficiary,
			} => AssetsAPIOf::<T, E>::mint(ext, asset.clone(), *amount, beneficiary).map(|v| v.encode()),
			AssetsApiInfo::Burn { asset, amount, who } => {
				AssetsAPIOf::<T, E>::burn(ext, asset.clone(), *amount, who).map(|v| v.encode())
			}
		}
	}
}
