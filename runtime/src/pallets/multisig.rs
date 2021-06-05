use super::error::Error;
use codec::{Decode, Encode};
use frame_support::weights::Weight;
use sp_core::hashing::blake2_256;
use substrate_subxt::{balances::Balances, Call, Client, Runtime};

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, Default, Debug)]
pub struct Timepoint<BlockNumber> {
    /// The height of the chain at the point in time.
    pub height: BlockNumber,
    /// The index of the extrinsic at the point in time.
    pub index: u32,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, Debug)]
pub struct MultisigData<BlockNumber, Balance, AccountId> {
    /// The extrinsic when the multisig operation was opened.
    pub when: Timepoint<BlockNumber>,
    /// The amount held in reserve of the `depositor`, to be returned once the operation ends.
    pub deposit: Balance,
    /// The account who opened it (i.e. the first to approve it).
    pub depositor: AccountId,
    /// The approvals achieved so far, including the depositor. Always sorted.
    pub approvals: Vec<AccountId>,
}

#[derive(Encode, Copy, Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Store)]
pub struct MultisigsStore<T: Multisig> {
    #[store(returns = Option<MultisigData<T::BlockNumber,T::Balance, T::AccountId>>)]
    pub multisig_account: T::AccountId,
    pub call_hash: [u8; 32],
}

impl<BlockNumber> Timepoint<BlockNumber> {
    pub fn new(height: BlockNumber, index: u32) -> Self {
        Timepoint { height, index }
    }
}

#[module]
pub trait Multisig: Balances {}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct AsMultiCall<T: Multisig> {
    pub threshold: u16,
    pub other_signatories: Vec<T::AccountId>,
    pub maybe_timepoint: Option<Timepoint<T::BlockNumber>>,
    pub call: Vec<u8>,
    pub store_call: bool,
    pub max_weight: Weight,
}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct ApproveAsMultiCall<T: Multisig> {
    pub threshold: u16,
    pub other_signatories: Vec<T::AccountId>,
    pub maybe_timepoint: Option<Timepoint<T::BlockNumber>>,
    pub call_hash: [u8; 32],
    pub max_weight: Weight,
}

///////////////////////////////////////////
pub fn multisig_approve_as_multi_call<T: Multisig + Runtime, C: Call<T> + Send + Sync>(
    subxt_client: &Client<T>,
    threshold: u16,
    other_signatories: Vec<T::AccountId>,
    maybe_timepoint: Option<Timepoint<T::BlockNumber>>,
    call: C,
    max_weight: Weight,
) -> Result<ApproveAsMultiCall<T>, Error> {
    let call_encoded = subxt_client
        .encode(call)
        .map_err(|e| Error::SubxtError(e))?;
    let call_hash = blake2_256(&call_encoded.encode());
    Ok(ApproveAsMultiCall::<T> {
        threshold,
        other_signatories,
        maybe_timepoint,
        call_hash,
        max_weight,
    })
}

pub fn multisig_as_multi_call<T: Multisig + Runtime, C: Call<T> + Send + Sync>(
    subxt_client: &Client<T>,
    threshold: u16,
    other_signatories: Vec<T::AccountId>,
    maybe_timepoint: Option<Timepoint<T::BlockNumber>>,
    call: C,
    store_call: bool,
    max_weight: Weight,
) -> Result<AsMultiCall<T>, Error> {
    let call_encoded = subxt_client
        .encode(call)
        .map_err(|e| Error::SubxtError(e))?;
    let call = call_encoded.encode();
    Ok(AsMultiCall::<T> {
        threshold,
        other_signatories,
        maybe_timepoint,
        call,
        store_call,
        max_weight,
    })
}

pub fn multisig_call_hash<T: Multisig + Runtime, C: Call<T> + Send + Sync>(
    subxt_client: &Client<T>,
    call: C,
) -> Result<[u8; 32], Error> {
    let call_encoded = subxt_client
        .encode(call)
        .map_err(|e| Error::SubxtError(e))?;
    let call_hash = blake2_256(&call_encoded.encode());
    Ok(call_hash)
}
