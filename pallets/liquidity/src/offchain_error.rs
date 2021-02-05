use core::fmt;

pub enum OffchainError {
    NoAccountAvailable,
    Other(&'static str),
    TooEarlyToSendUnsignedTransaction,
}

impl fmt::Debug for OffchainError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NoAccountAvailable => write!(f, "No account available"),
            Self::Other(err) => write!(f, "{}", err),
            Self::TooEarlyToSendUnsignedTransaction => {
                write!(f, "Too early to send unsigned transaction")
            }
        }
    }
}

impl fmt::Display for OffchainError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OffchainError {}
