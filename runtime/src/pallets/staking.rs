use substrate_subxt::staking::{BondCall, NominateCall, RewardDestination, Staking};

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
