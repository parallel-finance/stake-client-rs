use codec::Encode;
pub use substrate_subxt::staking::BondedStore;
use substrate_subxt::staking::{
    BondCall, NominateCall, RewardDestination, Staking as SubxtStaking,
};

#[module]
pub trait Staking: SubxtStaking {}

#[derive(Call, Encode, Debug, Clone)]
pub struct BondExtraCall<T: Staking> {
    #[codec(compact)]
    pub max_additional: T::Balance,
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
