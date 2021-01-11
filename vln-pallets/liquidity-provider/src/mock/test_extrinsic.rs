//! Testing utilities.

use core::fmt;
use parity_scale_codec::{Codec, Decode, Encode};
use sp_runtime::{
    traits::{
        Applyable, Checkable, DispatchInfoOf, Dispatchable, Extrinsic, PostDispatchInfoOf,
        SignedExtension, ValidateUnsigned,
    },
    transaction_validity::{TransactionSource, TransactionValidity, TransactionValidityError},
    ApplyExtrinsicResultWithInfo,
};

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
pub struct TestXt<AccountId, Call, Extra> {
    pub call: Call,
    pub signature: Option<(AccountId, Extra)>,
}

impl<AccountId, Call, Extra, Origin> Applyable for TestXt<AccountId, Call, Extra>
where
    AccountId: Send + Sync,
    Call: 'static
        + Clone
        + Codec
        + Dispatchable<Origin = Origin>
        + Eq
        + fmt::Debug
        + Send
        + Sized
        + Sync,
    Extra: SignedExtension<AccountId = AccountId, Call = Call>,
    Origin: From<Option<AccountId>>,
{
    type Call = Call;

    fn validate<U: ValidateUnsigned<Call = Self::Call>>(
        &self,
        _source: TransactionSource,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        Ok(Default::default())
    }

    fn apply<U: ValidateUnsigned<Call = Self::Call>>(
        self,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> ApplyExtrinsicResultWithInfo<PostDispatchInfoOf<Self::Call>> {
        let maybe_who = if let Some((who, extra)) = self.signature {
            Extra::pre_dispatch(extra, &who, &self.call, info, len)?;
            Some(who)
        } else {
            Extra::pre_dispatch_unsigned(&self.call, info, len)?;
            None
        };

        Ok(self.call.dispatch(maybe_who.into()))
    }
}

impl<AccountId, Call, Context, Extra> Checkable<Context> for TestXt<AccountId, Call, Extra>
where
    Call: Codec + Sync + Send,
{
    type Checked = Self;
    fn check(self, _: &Context) -> Result<Self::Checked, TransactionValidityError> {
        Ok(self)
    }
}

impl<AccountId, Call, Extra> Extrinsic for TestXt<AccountId, Call, Extra>
where
    Call: Codec + Sync + Send,
{
    type Call = Call;
    type SignaturePayload = (AccountId, Extra);

    fn is_signed(&self) -> Option<bool> {
        Some(self.signature.is_some())
    }

    fn new(c: Call, sig: Option<Self::SignaturePayload>) -> Option<Self> {
        Some(TestXt {
            signature: sig,
            call: c,
        })
    }
}

#[cfg(feature = "serde")]
impl<AccountId, Call, Extra> Serialize for TestXt<AccountId, Call, Extra>
where
    TestXt<Call, Extra>: Encode,
{
    fn serialize<S>(&self, seq: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.using_encoded(|bytes| seq.serialize_bytes(bytes))
    }
}

impl<AccountId, Call, Extra> fmt::Debug for TestXt<AccountId, Call, Extra>
where
    AccountId: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TestXt({:?}, ...)",
            self.signature.as_ref().map(|x| &x.0)
        )
    }
}

parity_util_mem::malloc_size_of_is_0!(any: TestXt<AccountId, Call, Extra>);
