use codec::{Decode, Encode};
use frame_support::{pallet_prelude::Member, BoundedVec, Parameter};
use substrate_subxt::balances::Balances;

#[module]
pub trait NomineeElection: Balances {
    type MaxValidators: Parameter + Member;
}

/// Info of the validator to be elected
#[derive(Encode, Decode, Eq, PartialEq, Clone, Default)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub struct ValidatorInfo<T: NomineeElection> {
    pub name: Option<Vec<u8>>,
    // Account Id
    pub address: T::AccountId,
    // Nomination (token amount)
    pub stakes: u128,
    // Score
    pub score: u128,
}

pub type ValidatorSet<T> = BoundedVec<ValidatorInfo<T>, <T as NomineeElection>::MaxValidators>;
