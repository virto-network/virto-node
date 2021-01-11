use sp_arithmetic::traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};

pub trait LiquidityProviderBalance:
    CheckedAdd + CheckedDiv + CheckedMul + CheckedSub + From<u32>
{
}

impl<T> LiquidityProviderBalance for T where
    T: CheckedAdd + CheckedDiv + CheckedMul + CheckedSub + From<u32>
{
}
