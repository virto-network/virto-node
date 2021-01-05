use core::{fmt, marker::PhantomData};
use sp_runtime::{
    generic::Era, traits::SignedExtension, transaction_validity::TransactionValidityError,
};
use substrate_subxt::{balances::Balances, system::System, SignedExtra};

type ExtraTy<T> = (
    CheckSpecVersion<T>,
    CheckTxVersion<T>,
    CheckGenesis<T>,
    CheckEra<T>,
    CheckNonce<T>,
    CheckWeight<T>,
);

#[derive(Clone, Debug, Eq, PartialEq, parity_scale_codec::Decode, parity_scale_codec::Encode)]
pub struct ValiuExtra<T: System> {
    genesis_hash: T::Hash,
    nonce: T::Index,
    spec_version: u32,
    tx_version: u32,
}

impl<T> SignedExtra<T> for ValiuExtra<T>
where
    T: Balances + Clone + Eq + Send + Sync + System + fmt::Debug,
{
    type Extra = ExtraTy<T>;

    fn new(spec_version: u32, tx_version: u32, nonce: T::Index, genesis_hash: T::Hash) -> Self {
        Self {
            spec_version,
            tx_version,
            nonce,
            genesis_hash,
        }
    }

    fn extra(&self) -> Self::Extra {
        (
            CheckSpecVersion(PhantomData, self.spec_version),
            CheckTxVersion(PhantomData, self.tx_version),
            CheckGenesis(PhantomData, self.genesis_hash),
            CheckEra((Era::Immortal, PhantomData), self.genesis_hash),
            CheckNonce(self.nonce),
            CheckWeight(PhantomData),
        )
    }
}

impl<T> SignedExtension for ValiuExtra<T>
where
    T: Balances + Clone + Eq + Send + Sync + System + fmt::Debug,
{
    const IDENTIFIER: &'static str = "ValiuExtra";

    type AccountId = T::AccountId;
    type AdditionalSigned = <<Self as SignedExtra<T>>::Extra as SignedExtension>::AdditionalSigned;
    type Call = ();
    type Pre = ();

    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
        self.extra().additional_signed()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, parity_scale_codec::Decode, parity_scale_codec::Encode)]
pub struct CheckSpecVersion<T: System>(pub PhantomData<T>, #[codec(skip)] pub u32);

impl<T> SignedExtension for CheckSpecVersion<T>
where
    T: Clone + Eq + Send + Sync + System + fmt::Debug,
{
    const IDENTIFIER: &'static str = "CheckSpecVersion";

    type AccountId = u64;
    type AdditionalSigned = u32;
    type Call = ();
    type Pre = ();

    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
        Ok(self.1)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, parity_scale_codec::Decode, parity_scale_codec::Encode)]
pub struct CheckTxVersion<T: System>(pub PhantomData<T>, #[codec(skip)] pub u32);

impl<T> SignedExtension for CheckTxVersion<T>
where
    T: Clone + Eq + Send + Sync + System + fmt::Debug,
{
    const IDENTIFIER: &'static str = "CheckTxVersion";

    type AccountId = u64;
    type AdditionalSigned = u32;
    type Call = ();
    type Pre = ();

    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
        Ok(self.1)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, parity_scale_codec::Decode, parity_scale_codec::Encode)]
pub struct CheckGenesis<T: System>(pub PhantomData<T>, #[codec(skip)] pub T::Hash);

impl<T> SignedExtension for CheckGenesis<T>
where
    T: Clone + Eq + Send + Sync + System + fmt::Debug,
{
    const IDENTIFIER: &'static str = "CheckGenesis";

    type AccountId = u64;
    type AdditionalSigned = T::Hash;
    type Call = ();
    type Pre = ();

    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
        Ok(self.1)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, parity_scale_codec::Decode, parity_scale_codec::Encode)]
pub struct CheckEra<T: System>(pub (Era, PhantomData<T>), #[codec(skip)] pub T::Hash);

impl<T> SignedExtension for CheckEra<T>
where
    T: Clone + Eq + Send + Sync + System + fmt::Debug,
{
    const IDENTIFIER: &'static str = "CheckEra";

    type AccountId = u64;
    type AdditionalSigned = T::Hash;
    type Call = ();
    type Pre = ();

    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
        Ok(self.1)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, parity_scale_codec::Decode, parity_scale_codec::Encode)]
pub struct CheckNonce<T: System>(#[codec(compact)] pub T::Index);

impl<T> SignedExtension for CheckNonce<T>
where
    T: Clone + Eq + Send + Sync + System + fmt::Debug,
{
    const IDENTIFIER: &'static str = "CheckNonce";

    type AccountId = u64;
    type AdditionalSigned = ();
    type Call = ();
    type Pre = ();

    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, parity_scale_codec::Decode, parity_scale_codec::Encode)]
pub struct CheckWeight<T: System>(pub PhantomData<T>);

impl<T> SignedExtension for CheckWeight<T>
where
    T: Clone + Eq + Send + Sync + System + fmt::Debug,
{
    const IDENTIFIER: &'static str = "CheckWeight";

    type AccountId = u64;
    type AdditionalSigned = ();
    type Call = ();
    type Pre = ();

    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
        Ok(())
    }
}
