use codec::{Decode, Encode};
pub use substrate_subxt::staking::BondedStore;
use substrate_subxt::staking::{
    BondCall, NominateCall, RewardDestination, Staking as SubxtStaking,
};
pub use substrate_subxt::system::System;

/// Reward event.
#[derive(Clone, Debug, Eq, PartialEq, Event, Decode)]
pub struct RewardEvent<T: Staking> {
    /// Account balance was transfered from.
    pub account: T::AccountId,
    /// Amount of balance that was transfered.
    pub amount: T::Balance,
}

/// Slash event.
#[derive(Clone, Debug, Eq, PartialEq, Event, Decode)]
pub struct SlashEvent<T: Staking> {
    /// Account balance was transfered from.
    pub account: T::AccountId,
    /// Amount of balance that was transfered.
    pub amount: T::Balance,
}

#[module]
pub trait Staking: SubxtStaking {}

#[derive(Call, Encode, Debug, Clone)]
pub struct BondExtraCall<T: Staking> {
    #[codec(compact)]
    pub max_additional: T::Balance,
}

#[derive(Call, Encode, Debug, Clone)]
pub struct UnbondCall<T: Staking> {
    #[codec(compact)]
    pub value: T::Balance,
}

pub fn staking_bond_call<'a, T: Staking>(
    controller: &'a T::Address,
    value: T::Balance,
    payee: RewardDestination<T::AccountId>,
) -> BondCall<T> {
    BondCall::<T> {
        controller,
        value,
        payee,
    }
}

pub fn staking_nominate_call<'a, T: Staking>(targets: Vec<T::Address>) -> NominateCall<T> {
    NominateCall::<T> { targets }
}

pub fn staking_bond_extra_call<T: Staking>(max_additional: T::Balance) -> BondExtraCall<T> {
    BondExtraCall::<T> { max_additional }
}

pub fn staking_unbond_call<T: Staking>(value: T::Balance) -> UnbondCall<T> {
    UnbondCall::<T> { value }
}
