use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    MultiSignature,
};

/// Alias to type for a signature for a transaction on the relay chain. This allows one of several
/// kinds of underlying crypto to be used, so isn't a fixed size when encoded.
pub type Signature = MultiSignature;

/// Alias to the public key used for this chain, actually a `MultiSigner`. Like the signature, this
/// also isn't a fixed size when encoded, as different cryptos have different size public keys.
pub type AccountPublic = <Signature as Verify>::Signer;

/// Alias to the opaque account ID type for this chain, actually a `AccountId32`. This is always
/// 32 bytes.
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

/// The minimum balance of pool to withdraw.
pub const MIN_WITHDRAW_BALANCE: u128 = 100_000_000_000_000;

/// The maximum balance of pool to withdraw.
pub const MAX_WITHDRAW_BALANCE: u128 = 1000_000_000_000_000;

/// The tasks type
pub enum TasksType {
    ParaStake(Amount),
    ParaUnstake(AccountId, Amount),
    RelayUnbonded(AccountId, Amount),
    RelayWithdrawUnbonded(AccountId, Amount),
    RelayEraIndexChanged(u32),
}
pub type Amount = u128;
