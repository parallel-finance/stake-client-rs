use super::kusama;
use super::AccountId;
use super::Error;
use super::KusamaRuntime;
use super::MIN_POOL_BALANCE;
use log::{error, info, warn};
use sp_core::crypto::Ss58Codec;
use sp_keyring::AccountKeyring;
use std::str::FromStr;
use substrate_subxt::{staking, Client, Signer};
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
        MIN_POOL_BALANCE,
        staking::RewardDestination::Staked,
    );

    let account_id = AccountId::from_string(&pool_addr)
        .map_err(|_e| Error::Other("parse pool_addr to account id error".to_string()))?;

    // check again if the balance is correct
    let _ = check_balance(subxt_client, account_id.clone()).await?;

    let call_hash = kusama::api::multisig_call_hash(subxt_client, call)
        .map_err(|e| Error::ClientRuntimeError(e))?;
    let when = get_time_point(subxt_client, account_id.clone(), call_hash).await;
    if let Some(_) = when {
        warn!("timepoint {:?} exists, multisig already initial", when);
        return Err(Error::Other("timepoint exists".to_string()));
    }
    info!("multisig timepoint: {:?}", when);

    // FIXME, implement clone call
    let call = kusama::api::staking_bond_call::<KusamaRuntime>(
        &ctrl,
        MIN_POOL_BALANCE,
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
            if free - misc_frozen >= MIN_POOL_BALANCE {
                info!("can initial new multisig");
                return Some(());
            }
            None
        })
        .ok_or(Error::Other(
            "free - misc_frozen < MIN_POOL_BALANCE, cann't initial new multisig".to_string(),
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
        MIN_POOL_BALANCE,
        staking::RewardDestination::Staked,
    );
    let public = sp_core::ed25519::Public::from_str(&pool_addr)
        .map_err(|_e| Error::Other("parse pool_addr to account id error".to_string()))?;
    let call_hash = kusama::api::multisig_call_hash(subxt_client, call)
        .map_err(|e| Error::ClientRuntimeError(e))?;
    let when = get_time_point(subxt_client, public.into(), call_hash).await;
    if None == when {
        warn!("timepoint is null, multisig must initial first");
        return Err(Error::Other("timepoint is null".to_string()));
    }
    info!("multisig timepoint: {:?}", when);

    // FIXME: clone call
    let call = kusama::api::staking_bond_call::<KusamaRuntime>(
        &ctrl,
        MIN_POOL_BALANCE,
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

pub(crate) async fn get_time_point(
    subxt_client: &Client<KusamaRuntime>,
    multisig_account: AccountId,
    call_hash: [u8; 32],
) -> Option<kusama::api::Timepoint<u32>> {
    info!("get time point");
    let store = kusama::api::MultisigsStore::<KusamaRuntime> {
        multisig_account,
        call_hash,
    };
    if let Some(Some(kusama::api::MultisigData { when, .. })) = subxt_client
        .fetch(&store, None)
        .await
        .map_err(|e| error!("error get_time_point: {:?}", e))
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
    let call = kusama::api::staking_bond_extra_call::<KusamaRuntime>(MIN_POOL_BALANCE);

    let account_id = AccountId::from_string(&pool_addr)
        .map_err(|_e| Error::Other("parse pool_addr to account id error".to_string()))?;
    // check again if the balance is correct
    let _ = check_balance(subxt_client, account_id.clone()).await?;
    let call_hash = kusama::api::multisig_call_hash(subxt_client, call.clone())
        .map_err(|e| Error::ClientRuntimeError(e))?;
    let when = get_time_point(subxt_client, account_id.clone(), call_hash).await;
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
    let call = kusama::api::staking_bond_extra_call::<KusamaRuntime>(MIN_POOL_BALANCE);
    let public = sp_core::ed25519::Public::from_str(&pool_addr)
        .map_err(|_e| Error::Other("parse pool_addr to account id error".to_string()))?;
    let call_hash = kusama::api::multisig_call_hash(subxt_client, call.clone())
        .map_err(|e| Error::ClientRuntimeError(e))?;
    let when = get_time_point(subxt_client, public.into(), call_hash).await;
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
