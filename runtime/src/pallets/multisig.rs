use codec::{Encode,Decode};
use frame_support::{Parameter,weights::Weight};
use sp_runtime::traits::Member;
use substrate_subxt::system::System;
use sp_core::hashing::blake2_256;
use substrate_subxt::{Client,Encoded,Runtime,Call};
use super::error::Error;

use sp_runtime::traits::Hash;
#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, Default,Debug)]
pub struct Timepoint<BlockNumber> {
	/// The height of the chain at the point in time.
	height: BlockNumber,
	/// The index of the extrinsic at the point in time.
	index: u32,
}

impl<BlockNumber> Timepoint<BlockNumber> {
    pub fn new(height:BlockNumber, index:u32) ->Self {
        Timepoint {
            height,
            index,
        }
    }
}

#[module]
pub trait Multisig: System {}

#[derive(Clone, Debug, PartialEq, Call, Encode,Default)]
pub struct AsMultiCall<T: Multisig> {
    pub threshold: u16,
    pub other_signatories: Vec<T::AccountId>,
    pub maybe_timepoint: Option<Timepoint<T::BlockNumber>>,
    pub call: Vec<u8>,
    pub store_call: bool,
    pub max_weight: Weight,
}

#[derive(Clone, Debug, PartialEq, Call, Encode,Default)]
pub struct ApproveAsMultiCall<T: Multisig> {
    pub threshold: u16,
	pub other_signatories: Vec<T::AccountId>,
	pub maybe_timepoint: Option<Timepoint<T::BlockNumber>>,
	pub call_hash: [u8; 32],
	pub max_weight: Weight,
}


///////////////////////////////////////////
pub fn multisig_approve_as_multi_call<T: Multisig +Runtime,C: Call<T> + Send + Sync>(
    subxt_client: &Client<T>,
    threshold: u16,
    other_signatories: Vec<T::AccountId>,
    maybe_timepoint: Option<Timepoint<T::BlockNumber>>,
    call: C,
    max_weight: Weight,
)-> Result<ApproveAsMultiCall<T>,Error> {
    let call_encoded = subxt_client.encode(call).map_err(|e|Error::SubxtError(e))?;
    let call_hash = blake2_256(&call_encoded.encode());
    Ok(ApproveAsMultiCall::<T>{
        threshold,
        other_signatories,
        maybe_timepoint,
        call_hash,
        max_weight,
    })
}

pub fn multisig_as_multi_call<T: Multisig+Runtime,C: Call<T> + Send + Sync>(
    subxt_client: &Client<T>,
    threshold: u16,
    other_signatories: Vec<T::AccountId>,
    maybe_timepoint: Option<Timepoint<T::BlockNumber>>,
    call: C,
    store_call: bool,
    max_weight: Weight,
)-> Result<AsMultiCall<T>,Error> {
    let call_encoded = subxt_client.encode(call).map_err(|e|Error::SubxtError(e))?;
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