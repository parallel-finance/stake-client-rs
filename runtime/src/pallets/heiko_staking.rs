use codec::Encode;
use core::marker::PhantomData;
use substrate_subxt::system::System;
pub type Balance = u128;

#[module]
pub trait Staking: System {}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct StakeCall<T: Staking> {
    pub amount: Balance,
    pub _runtime: PhantomData<T>,
}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct WithdrawCall<T: Staking> {
    pub agent: T::AccountId,
    pub amount: Balance,
}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct RecordRewardsCall<T: Staking> {
    pub agent: T::AccountId,
    pub amount: Balance,
}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct RecordSlashCall<T: Staking> {
    pub agent: T::AccountId,
    pub amount: Balance,
}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct UnstakeCall<T: Staking> {
    pub amount: Balance,
    pub _runtime: PhantomData<T>,
}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct ProcessPendingUnstakeCall<T: Staking> {
    pub agent: T::AccountId,
    pub owner: T::AccountId,
    pub amount: Balance,
}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct FinishProcessedUnstakeCall<T: Staking> {
    agent: T::AccountId,
    owner: T::AccountId,
    amount: Balance,
}

pub fn staking_stake_call<'a, T: Staking>(amount: Balance) -> StakeCall<T> {
    StakeCall::<T> {
        amount,
        _runtime: PhantomData,
    }
}

pub fn staking_withdraw_call<'a, T: Staking>(
    agent: T::AccountId,
    amount: Balance,
) -> WithdrawCall<T> {
    WithdrawCall::<T> { agent, amount }
}

pub fn staking_record_rewards_call<'a, T: Staking>(
    agent: T::AccountId,
    amount: Balance,
) -> RecordRewardsCall<T> {
    RecordRewardsCall::<T> { agent, amount }
}

pub fn staking_record_slash_call<'a, T: Staking>(
    agent: T::AccountId,
    amount: Balance,
) -> RecordSlashCall<T> {
    RecordSlashCall::<T> { agent, amount }
}

pub fn staking_unstake_call<'a, T: Staking>(amount: Balance) -> UnstakeCall<T> {
    UnstakeCall::<T> {
        amount,
        _runtime: PhantomData,
    }
}

pub fn staking_process_pending_unstake_call<'a, T: Staking>(
    agent: T::AccountId,
    owner: T::AccountId,
    amount: Balance,
) -> ProcessPendingUnstakeCall<T> {
    ProcessPendingUnstakeCall::<T> {
        agent,
        owner,
        amount,
    }
}

pub fn staking_finish_processed_unstake_call<'a, T: Staking>(
    agent: T::AccountId,
    owner: T::AccountId,
    amount: Balance,
) -> FinishProcessedUnstakeCall<T> {
    FinishProcessedUnstakeCall::<T> {
        agent,
        owner,
        amount,
    }
}
