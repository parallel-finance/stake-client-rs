use codec::Encode;
use core::marker::PhantomData;
use substrate_subxt::system::System;
pub type Balance = u128;

#[module]
pub trait LiquidStaking: System {}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct StakeCall<T: LiquidStaking> {
    pub amount: Balance,
    pub _runtime: PhantomData<T>,
}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct WithdrawCall<T: LiquidStaking> {
    pub agent: T::AccountId,
    pub amount: Balance,
}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct RecordRewardsCall<T: LiquidStaking> {
    pub agent: T::AccountId,
    pub amount: Balance,
}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct RecordSlashCall<T: LiquidStaking> {
    pub agent: T::AccountId,
    pub amount: Balance,
}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct UnstakeCall<T: LiquidStaking> {
    pub amount: Balance,
    pub _runtime: PhantomData<T>,
}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct ProcessPendingUnstakeCall<T: LiquidStaking> {
    pub agent: T::AccountId,
    pub owner: T::AccountId,
    pub amount: Balance,
}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct FinishProcessedUnstakeCall<T: LiquidStaking> {
    agent: T::AccountId,
    owner: T::AccountId,
    amount: Balance,
}

pub fn liquid_staking_stake_call<'a, T: LiquidStaking>(amount: Balance) -> StakeCall<T> {
    StakeCall::<T> {
        amount,
        _runtime: PhantomData,
    }
}

pub fn liquid_staking_withdraw_call<'a, T: LiquidStaking>(
    agent: T::AccountId,
    amount: Balance,
) -> WithdrawCall<T> {
    WithdrawCall::<T> { agent, amount }
}

pub fn liquid_staking_record_rewards_call<'a, T: LiquidStaking>(
    agent: T::AccountId,
    amount: Balance,
) -> RecordRewardsCall<T> {
    RecordRewardsCall::<T> { agent, amount }
}

pub fn liquid_staking_record_slash_call<'a, T: LiquidStaking>(
    agent: T::AccountId,
    amount: Balance,
) -> RecordSlashCall<T> {
    RecordSlashCall::<T> { agent, amount }
}

pub fn liquid_staking_unstake_call<'a, T: LiquidStaking>(amount: Balance) -> UnstakeCall<T> {
    UnstakeCall::<T> {
        amount,
        _runtime: PhantomData,
    }
}

pub fn liquid_staking_process_pending_unstake_call<'a, T: LiquidStaking>(
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

pub fn liquid_staking_finish_processed_unstake_call<'a, T: LiquidStaking>(
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
