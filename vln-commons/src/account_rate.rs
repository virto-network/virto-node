/// The balance of an account
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
    /// Creates a new instance from `account` and `rate`.
    #[inline]
    pub fn new(account: A, rate: B) -> Self {
        Self { account, rate }
    }

    /// Account
    #[inline]
    pub const fn account(&self) -> &A {
        &self.account
    }

    /// Rate
    #[inline]
    pub const fn rate(&self) -> &B {
        &self.rate
    }

    /// Mutable rate
    #[inline]
    pub fn rate_mut(&mut self) -> &mut B {
        &mut self.rate
    }
}
