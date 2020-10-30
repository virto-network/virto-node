#[derive(
    Clone,
    Debug,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
)]
pub struct AccountRate<A, B> {
    account: A,
    rate: B,
}

impl<A, B> AccountRate<A, B> {
    #[inline]
    pub fn new(account: A, rate: B) -> Self {
        Self { account, rate }
    }

    #[inline]
    pub const fn account(&self) -> &A {
        &self.account
    }

    #[inline]
    pub const fn rate(&self) -> &B {
        &self.rate
    }

    #[inline]
    pub fn rate_mut(&mut self) -> &mut B {
        &mut self.rate
    }
}
