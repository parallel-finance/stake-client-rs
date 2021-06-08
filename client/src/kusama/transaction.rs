use super::heiko;
use super::kusama;
use super::AccountId;
use super::Amount;
use super::Error;
use super::HeikoRuntime;
use super::KusamaRuntime;
use super::Multisig;
use super::MIN_BOND_BALANCE;
use async_std::task;
use core::marker::PhantomData;
use log::{info, warn};
use runtime::pallets::liquid_staking::{RecordRewardsCall, RecordSlashCall};
use runtime::pallets::multisig::Timepoint;
use sp_core::crypto::Ss58Codec;
use sp_keyring::AccountKeyring;
use std::str::FromStr;
use std::time::Duration;
use substrate_subxt::ExtrinsicSuccess;
use substrate_subxt::{staking, sudo, Call, Client, Runtime, Signer};
/// The first wallet to call withdraw. No need use 'TimePoint' and call 'approve_as_multi'.
pub(crate) async fn do_first_relay_bond(
    others: Vec<AccountId>,
    pool_addr: String,
    subxt_client: &Client<KusamaRuntime>,
    signer: &(dyn Signer<KusamaRuntime> + Send + Sync),
) -> Result<(), Error> {
    info!("do_first_relay_bond");
    // 1.1 construct staking bond call
    //TODO change to controller address, change the payee type
    let ctrl = AccountKeyring::Eve.to_account_id().into();
    let call = kusama::api::staking_bond_call::<KusamaRuntime>(
        &ctrl,
        MIN_BOND_BALANCE,
        staking::RewardDestination::Staked,
    );

    let account_id = AccountId::from_string(&pool_addr)
        .map_err(|_e| Error::Other("parse pool_addr to account id error".to_string()))?;

    // check again if the balance is correct
    let _ = check_balance(subxt_client, account_id.clone()).await?;

    let call_hash = kusama::api::multisig_call_hash(subxt_client, call)
        .map_err(|e| Error::ClientRuntimeError(e))?;
    let when = get_time_point::<KusamaRuntime>(subxt_client, account_id.clone(), call_hash).await;
    if let Some(_) = when {
        warn!("timepoint {:?} exists, multisig already initial", when);
        return Err(Error::Other("timepoint exists".to_string()));
    }
    info!("multisig timepoint: {:?}", when);

    // FIXME, implement clone call
    let call = kusama::api::staking_bond_call::<KusamaRuntime>(
        &ctrl,
        MIN_BOND_BALANCE,
        staking::RewardDestination::Staked,
    );

    let mc = kusama::api::multisig_approve_as_multi_call::<
        KusamaRuntime,
        staking::BondCall<KusamaRuntime>,
    >(subxt_client, 2, others, None, call, 0u64)
    .map_err(|e| Error::ClientRuntimeError(e))?;
    // 1.2 initial the multisg call
    let result = subxt_client
        .watch(mc, signer)
        .await
        .map_err(|e| Error::SubxtError(e))?;
    info!("do_first_relay_bond result: {:?}", result);
    Ok(())
}

async fn check_balance(
    subxt_client: &Client<KusamaRuntime>,
    account_id: AccountId,
) -> Result<(), Error> {
    let account = kusama::api::AccountStore::<KusamaRuntime> {
        account: account_id,
    };
    subxt_client
        .fetch(&account, None)
        .await
        .map_err(|e| Error::SubxtError(e))?
        .and_then(|account_store| -> Option<()> {
            let free = account_store.data.free;
            let misc_frozen = account_store.data.misc_frozen;
            if free - misc_frozen >= MIN_BOND_BALANCE {
                info!("can initial new multisig");
                return Some(());
            }
            None
        })
        .ok_or(Error::Other(
            "free - misc_frozen < MIN_BOND_BALANCE, cann't initial new multisig".to_string(),
        ))
}

/// If the wallet is the last one need to get 'TimePoint' and call 'as_multi'.
pub(crate) async fn do_last_relay_bond(
    others: Vec<AccountId>,
    pool_addr: String,
    subxt_client: &Client<KusamaRuntime>,
    signer: &(dyn Signer<KusamaRuntime> + Send + Sync),
) -> Result<(), Error> {
    info!("do_last_relay_bond");
    let ctrl = AccountKeyring::Eve.to_account_id().into();
    let call = kusama::api::staking_bond_call::<KusamaRuntime>(
        &ctrl,
        MIN_BOND_BALANCE,
        staking::RewardDestination::Staked,
    );
    let public = sp_core::ed25519::Public::from_str(&pool_addr)
        .map_err(|_e| Error::Other("parse pool_addr to account id error".to_string()))?;
    let call_hash = kusama::api::multisig_call_hash(subxt_client, call)
        .map_err(|e| Error::ClientRuntimeError(e))?;
    let when = get_time_point::<KusamaRuntime>(subxt_client, public.into(), call_hash).await;
    if None == when {
        warn!("timepoint is null, multisig must initial first");
        return Err(Error::Other("timepoint is null".to_string()));
    }
    info!("multisig timepoint: {:?}", when);

    // FIXME: clone call
    let call = kusama::api::staking_bond_call::<KusamaRuntime>(
        &ctrl,
        MIN_BOND_BALANCE,
        staking::RewardDestination::Staked,
    );
    // 3.1 approve the call and execute it
    let mc = kusama::api::multisig_as_multi_call::<KusamaRuntime, staking::BondCall<KusamaRuntime>>(
        subxt_client,
        2,
        others,
        when,
        call,
        false,
        1_000_000_000_000,
    )?;
    let result = subxt_client
        .watch(mc, signer)
        .await
        .map_err(|e| Error::SubxtError(e))?;
    info!("multisig_as_multi_call result {:?}", result);
    Ok(())
}

pub(crate) async fn get_time_point<T: Runtime + Multisig>(
    subxt_client: &Client<T>,
    multisig_account: T::AccountId,
    call_hash: [u8; 32],
) -> Option<kusama::api::Timepoint<T::BlockNumber>> {
    info!("get time point");
    let store = kusama::api::MultisigsStore::<T> {
        multisig_account,
        call_hash,
    };
    if let Some(Some(kusama::api::MultisigData { when, .. })) = subxt_client
        .fetch(&store, None)
        .await
        .map_err(|e| warn!("error get_time_point: {:?}", e))
        .ok()
    {
        return Some(when);
    } else {
        warn!("warn get_time_point: None");
        return None;
    }
}

pub(crate) async fn do_first_relay_bond_extra(
    others: Vec<AccountId>,
    pool_addr: String,
    subxt_client: &Client<KusamaRuntime>,
    signer: &(dyn Signer<KusamaRuntime> + Send + Sync),
) -> Result<(), Error> {
    info!("do_first_relay_bond_extra");
    // 1.1 construct staking bond call
    let call = kusama::api::staking_bond_extra_call::<KusamaRuntime>(MIN_BOND_BALANCE);

    let account_id = AccountId::from_string(&pool_addr)
        .map_err(|_e| Error::Other("parse pool_addr to account id error".to_string()))?;
    // check again if the balance is correct
    let _ = check_balance(subxt_client, account_id.clone()).await?;
    let call_hash = kusama::api::multisig_call_hash(subxt_client, call.clone())
        .map_err(|e| Error::ClientRuntimeError(e))?;
    let when = get_time_point::<KusamaRuntime>(subxt_client, account_id.clone(), call_hash).await;
    if let Some(_) = when {
        warn!("timepoint {:?} exists, multisig already initial", when);
        return Err(Error::Other("timepoint exists".to_string()));
    }
    info!("multisig timepoint: {:?}", when);

    let mc = kusama::api::multisig_approve_as_multi_call::<
        KusamaRuntime,
        kusama::api::BondExtraCall<KusamaRuntime>,
    >(subxt_client, 2, others, None, call, 0u64)
    .map_err(|e| Error::ClientRuntimeError(e))?;
    // 1.2 initial the multisg call
    let result = subxt_client
        .watch(mc, signer)
        .await
        .map_err(|e| Error::SubxtError(e))?;
    info!("do_first_relay_bond result: {:?}", result);
    Ok(())
}

pub(crate) async fn do_last_relay_bond_extra(
    others: Vec<AccountId>,
    pool_addr: String,
    subxt_client: &Client<KusamaRuntime>,
    signer: &(dyn Signer<KusamaRuntime> + Send + Sync),
) -> Result<(), Error> {
    info!("do_last_relay_bond_extra");
    let call = kusama::api::staking_bond_extra_call::<KusamaRuntime>(MIN_BOND_BALANCE);
    let public = sp_core::ed25519::Public::from_str(&pool_addr)
        .map_err(|_e| Error::Other("parse pool_addr to account id error".to_string()))?;
    let call_hash = kusama::api::multisig_call_hash(subxt_client, call.clone())
        .map_err(|e| Error::ClientRuntimeError(e))?;
    let when = get_time_point::<KusamaRuntime>(subxt_client, public.into(), call_hash).await;
    if None == when {
        warn!("timepoint is null, multisig must initial first");
        return Err(Error::Other("timepoint is null".to_string()));
    }
    info!("multisig timepoint: {:?}", when);

    // 3.1 approve the call and execute it
    let mc = kusama::api::multisig_as_multi_call::<
        KusamaRuntime,
        kusama::api::BondExtraCall<KusamaRuntime>,
    >(
        subxt_client,
        2,
        others,
        when,
        call,
        false,
        1_000_000_000_000,
    )?;
    let result = subxt_client
        .watch(mc, signer)
        .await
        .map_err(|e| Error::SubxtError(e))?;
    info!("multisig_as_multi_call result {:?}", result);
    Ok(())
}

pub(crate) async fn do_first_para_record_rewards(
    others: Vec<AccountId>,
    pool_addr: String,
    subxt_client: &Client<HeikoRuntime>,
    signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    amount: Amount,
) -> Result<(), Error> {
    info!("do_first_para_record_rewards");
    // 1.1 construct inner call
    let account_id = AccountId::from_string(&pool_addr)
        .map_err(|_e| Error::Other("parse pool_addr to account id error".to_string()))?;
    let inner_call =
        heiko::api::liquid_staking_record_rewards_call::<HeikoRuntime>(account_id.clone(), amount);
    let result = first_para_record_reward_and_slash::<RecordRewardsCall<HeikoRuntime>>(
        others,
        account_id,
        subxt_client,
        signer,
        inner_call,
    )
    .await?;
    info!("do_first_para_record_rewards result: {:?}", result);
    Ok(())
}

pub(crate) async fn do_last_para_record_rewards(
    others: Vec<AccountId>,
    pool_addr: String,
    subxt_client: &Client<HeikoRuntime>,
    signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    amount: Amount,
) -> Result<(), Error> {
    info!("do_last_para_record_rewards");
    // 1.1 construct inner call
    let account_id = AccountId::from_string(&pool_addr)
        .map_err(|_e| Error::Other("parse pool_addr to account id error".to_string()))?;
    let inner_call =
        heiko::api::liquid_staking_record_rewards_call::<HeikoRuntime>(account_id.clone(), amount);
    let result = last_para_record_reward_and_slash::<RecordRewardsCall<HeikoRuntime>>(
        others,
        account_id,
        subxt_client,
        signer,
        inner_call,
    )
    .await?;
    info!("do_last_para_record_rewards result: {:?}", result);
    Ok(())
}

pub(crate) async fn do_first_para_record_slash(
    others: Vec<AccountId>,
    pool_addr: String,
    subxt_client: &Client<HeikoRuntime>,
    signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    amount: Amount,
) -> Result<(), Error> {
    info!("do_first_para_record_slash");
    let account_id = AccountId::from_string(&pool_addr)
        .map_err(|_e| Error::Other("parse pool_addr to account id error".to_string()))?;
    let inner_call =
        heiko::api::liquid_staking_record_slash_call::<HeikoRuntime>(account_id.clone(), amount);
    let result = first_para_record_reward_and_slash::<RecordSlashCall<HeikoRuntime>>(
        others,
        account_id,
        subxt_client,
        signer,
        inner_call,
    )
    .await?;
    info!("do_first_para_record_slash result: {:?}", result);
    Ok(())
}

pub(crate) async fn do_last_para_record_slash(
    others: Vec<AccountId>,
    pool_addr: String,
    subxt_client: &Client<HeikoRuntime>,
    signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    amount: Amount,
) -> Result<(), Error> {
    info!("do_last_para_record_slash");
    // 1.1 construct inner call
    let account_id = AccountId::from_string(&pool_addr)
        .map_err(|_e| Error::Other("parse pool_addr to account id error".to_string()))?;
    let inner_call =
        heiko::api::liquid_staking_record_slash_call::<HeikoRuntime>(account_id.clone(), amount);
    let result = last_para_record_reward_and_slash::<RecordSlashCall<HeikoRuntime>>(
        others,
        account_id,
        subxt_client,
        signer,
        inner_call,
    )
    .await?;
    info!("do_last_para_record_slash result: {:?}", result);
    Ok(())
}

// TODO try to integrate `first_para_record_reward_and_slash` and `last_para_record_reward_and_slash`
async fn first_para_record_reward_and_slash<C: Call<HeikoRuntime> + Send + Sync>(
    others: Vec<AccountId>,
    account_id: AccountId,
    subxt_client: &Client<HeikoRuntime>,
    signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    inner_call: C,
) -> Result<ExtrinsicSuccess<HeikoRuntime>, Error> {
    // 1.2 construct sudo call
    let inner_call_encoded = subxt_client
        .encode(inner_call)
        .map_err(|e| Error::SubxtError(e))?;
    let sudo_call = sudo::SudoCall::<HeikoRuntime> {
        _runtime: PhantomData,
        call: &inner_call_encoded,
    };

    // check if timepoint already exist.
    let call_hash = heiko::api::multisig_call_hash(subxt_client, sudo_call.clone())
        .map_err(|e| Error::ClientRuntimeError(e))?;
    //FIXME, multisig accout should change
    let when = get_time_point::<HeikoRuntime>(subxt_client, account_id.clone(), call_hash).await;
    if let Some(_) = when {
        warn!("timepoint {:?} exists, multisig already initial", when);
        return Err(Error::Other("timepoint exists".to_string()));
    }
    info!("multisig timepoint: {:?}", when);

    // 1.3 construct multisig call
    let multisig_call = heiko::api::multisig_approve_as_multi_call::<
        HeikoRuntime,
        sudo::SudoCall<HeikoRuntime>,
    >(subxt_client, 2, others, None, sudo_call, 0u64)
    .map_err(|e| Error::ClientRuntimeError(e))?;
    // 1.2 initial the multisg call
    let result = subxt_client
        .watch(multisig_call, signer)
        .await
        .map_err(|e| Error::SubxtError(e))?;
    Ok(result)
}

async fn last_para_record_reward_and_slash<C: Call<HeikoRuntime> + Send + Sync>(
    others: Vec<AccountId>,
    account_id: AccountId,
    subxt_client: &Client<HeikoRuntime>,
    signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    inner_call: C,
) -> Result<ExtrinsicSuccess<HeikoRuntime>, Error> {
    // 1.2 construct sudo call
    let inner_call_encoded = subxt_client
        .encode(inner_call)
        .map_err(|e| Error::SubxtError(e))?;
    let sudo_call = sudo::SudoCall::<HeikoRuntime> {
        _runtime: PhantomData,
        call: &inner_call_encoded,
    };

    // check if timepoint already exist.
    let call_hash = heiko::api::multisig_call_hash(subxt_client, sudo_call.clone())
        .map_err(|e| Error::ClientRuntimeError(e))?;

    //TODO this `loop` is really a temporary check way.
    let mut check_times = 0u8;
    let mut when: Option<Timepoint<u32>>;
    loop {
        if check_times == 3u8 {
            return Err(Error::Other("timepoint is null".to_string()));
        }
        //FIXME, multisig accout should change
        when = get_time_point::<HeikoRuntime>(subxt_client, account_id.clone(), call_hash.clone())
            .await;
        if None == when {
            check_times = check_times + 1u8;
            warn!(
                "timepoint is null, multisig must initial first {}",
                check_times
            );
            task::sleep(Duration::from_millis(12000)).await;
            continue;
        } else {
            break;
        }
    }

    info!("multisig timepoint: {:?}", when);

    // 1.3 construct multisig call
    let multisig_call =
        heiko::api::multisig_as_multi_call::<HeikoRuntime, sudo::SudoCall<HeikoRuntime>>(
            subxt_client,
            2,
            others,
            when,
            sudo_call,
            false,
            1_000_000_000_000,
        )
        .map_err(|e| Error::ClientRuntimeError(e))?;
    // 1.2 initial the multisg call
    let result = subxt_client
        .watch(multisig_call, signer)
        .await
        .map_err(|e| Error::SubxtError(e))?;
    Ok(result)
}
