// Swap Handler trait details how to perform swap between different 
// onchain and offchain combinations
// The flow of a HumanSwap can be as follows
// 1. CASH-OUT : Onchain Account -> Offchain Recipent (Onchain account creates a swap request and transfers assets to the Liquidity provider that proves the offchain transfer to recipent)
// 2. CASH-IN : Offchain Sender -> Onchain Account (The offchain sender transfers assets to a LP, and an onchain transfer is executed from the LP to onchain account)
// In both of the above cases a Liquidity provider is essential to ensure the transfer is completed and this is represented by the human (RateProvider).
// Maybe this SwapHandler can be more generic??
pub trait SwapHandler {
    // Create cash in - perform custom logic to see if cash in is possible?
    fn create_cash_in(who: Origin, human: RateProvider, amount: Balance, from_currency: Currency, to_currency: Currency) -> Result;
    // Create cash in - perform custom logic to see if cash out is possible?
    fn create_cash_out(who: Origin, human: RateProvider, amount: Balance, from_currency: Currency, to_currency: Currency) -> Result;
    // Complete cash in can be done by transferring the onchain balance by the RateProvider Origin
    fn complete_cash_in(who: Origin, amount: Balance, currency: Currency, dest: Destination) -> Result;
    // Complete cash out by uploading proof, nothing else to do at this point
    fn complete_cash_out(who: Origin, proof: FileHash) -> Result;
}

// We can make SwapHandler generic and put valiu business logic here
pub struct ValiuSwapHandler;
impl SwapHandler for ValiuSwapHandler {
}