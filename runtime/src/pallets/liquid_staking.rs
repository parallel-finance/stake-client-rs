use codec::{Decode, Encode};
use core::marker::PhantomData;
use substrate_subxt::balances::Balances;

/// Unstaked event.
#[derive(Clone, Debug, Eq, PartialEq, Event, Decode)]
pub struct UnstakedEvent<T: LiquidStaking> {
    /// Account who unstake.
    pub account: T::AccountId,
    /// voucher that will unstaked.
    pub voucher: T::Balance,
    /// Amount of balance that will unstaked.
    pub amount: T::Balance,
}

#[derive(Encode, Decode, Copy, Clone, Debug, Default, Store)]
pub struct TotalStakingAssetStore<T: LiquidStaking> {
    #[store(returns = T::Balance)]
    /// Marker for the runtime
    pub _runtime: PhantomData<T>,
}

#[derive(Encode, Decode, Copy, Clone, Debug, Default, Store)]
pub struct TotalVoucherStore<T: LiquidStaking> {
    #[store(returns = T::Balance)]
    /// Marker for the runtime
    pub _runtime: PhantomData<T>,
}

#[module]
pub trait LiquidStaking: Balances {}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct StakeCall<T: LiquidStaking> {
    pub amount: T::Balance,
    pub _runtime: PhantomData<T>,
}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct WithdrawCall<T: LiquidStaking> {
    pub agent: T::AccountId,
    pub amount: T::Balance,
}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct RecordRewardsCall<T: LiquidStaking> {
    pub agent: T::AccountId,
    pub amount: T::Balance,
}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct RecordSlashCall<T: LiquidStaking> {
    pub agent: T::AccountId,
    pub amount: T::Balance,
}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct UnstakeCall<T: LiquidStaking> {
    pub amount: T::Balance,
    pub _runtime: PhantomData<T>,
}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct ProcessPendingUnstakeCall<T: LiquidStaking> {
    pub agent: T::AccountId,
    pub owner: T::AccountId,
    pub era_index: u32,
    pub amount: T::Balance,
}

#[derive(Clone, Debug, PartialEq, Call, Encode, Default)]
pub struct FinishProcessedUnstakeCall<T: LiquidStaking> {
    agent: T::AccountId,
    owner: T::AccountId,
    amount: T::Balance,
}

pub fn liquid_staking_stake_call<'a, T: LiquidStaking>(amount: T::Balance) -> StakeCall<T> {
    StakeCall::<T> {
        amount,
        _runtime: PhantomData,
    }
}

pub fn liquid_staking_withdraw_call<'a, T: LiquidStaking>(
    agent: T::AccountId,
    amount: T::Balance,
) -> WithdrawCall<T> {
    WithdrawCall::<T> { agent, amount }
}

pub fn liquid_staking_record_rewards_call<'a, T: LiquidStaking>(
    agent: T::AccountId,
    amount: T::Balance,
) -> RecordRewardsCall<T> {
    RecordRewardsCall::<T> { agent, amount }
}

pub fn liquid_staking_record_slash_call<'a, T: LiquidStaking>(
    agent: T::AccountId,
    amount: T::Balance,
) -> RecordSlashCall<T> {
    RecordSlashCall::<T> { agent, amount }
}

pub fn liquid_staking_unstake_call<'a, T: LiquidStaking>(amount: T::Balance) -> UnstakeCall<T> {
    UnstakeCall::<T> {
        amount,
        _runtime: PhantomData,
    }
}

pub fn liquid_staking_process_pending_unstake_call<'a, T: LiquidStaking>(
    agent: T::AccountId,
    owner: T::AccountId,
    era_index: u32,
    amount: T::Balance,
) -> ProcessPendingUnstakeCall<T> {
    ProcessPendingUnstakeCall::<T> {
        agent,
        owner,
        era_index,
        amount,
    }
}

pub fn liquid_staking_finish_processed_unstake_call<'a, T: LiquidStaking>(
    agent: T::AccountId,
    owner: T::AccountId,
    amount: T::Balance,
) -> FinishProcessedUnstakeCall<T> {
    FinishProcessedUnstakeCall::<T> {
        agent,
        owner,
        amount,
    }
}
