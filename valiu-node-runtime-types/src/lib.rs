#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::{
    ChainContext, CheckEra, CheckGenesis, CheckNonce, CheckSpecVersion, CheckTxVersion, CheckWeight,
};
use sp_core::H256;
use sp_runtime::{
    generic,
    traits::{BlakeTwo256, IdentifyAccount, Verify},
    MultiSignature,
};

/// Account data
pub type AccountData = ();

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// The address format for describing accounts.
pub type Address = AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Block type as expected by the runtime.
pub type Block<C, R> = generic::Block<Header, UncheckedExtrinsic<C, R>>;

/// BlockId type as expected by the runtime.
pub type BlockId<C, R> = generic::BlockId<Block<C, R>>;

/// An index to a block.
pub type BlockNumber = u32;

/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic<C, R> = generic::CheckedExtrinsic<AccountId, C, SignedExtra<R>>;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

/// Executive: handles dispatch to the various modules.
pub type Executive<AM, C, R> = frame_executive::Executive<R, Block<C, R>, ChainContext<R>, R, AM>;

/// A hash of some data used by the chain.
pub type Hash = H256;

/// Hashing algorithm
pub type Hashing = BlakeTwo256;

/// Block header type as expected by the runtime.
pub type Header = generic::Header<BlockNumber, Hashing>;

/// Index of a transaction in the chain.
pub type Index = u32;

/// Opaque block type.
pub type OpaqueBlock = generic::Block<Header, OpaqueExtrinsic>;

/// Opaque block identifier type.
pub type OpaqueBlockId = generic::BlockId<OpaqueExtrinsic>;

/// Opaque extrinsic
pub type OpaqueExtrinsic = sp_runtime::OpaqueExtrinsic;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// A Block signed with a Justification
pub type SignedBlock<C, R> = generic::SignedBlock<Block<C, R>>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra<R> = (
    CheckSpecVersion<R>,
    CheckTxVersion<R>,
    CheckGenesis<R>,
    CheckEra<R>,
    CheckNonce<R>,
    CheckWeight<R>,
);

/// Unchecked extrinsic type as expected by the runtime.
pub type UncheckedExtrinsic<C, R> =
    generic::UncheckedExtrinsic<Address, C, Signature, SignedExtra<R>>;
