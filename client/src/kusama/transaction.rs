use super::AccountId;
use super::Error;
use super::KusamaRuntime;
use super::MIN_POOL_BALANCE;
use log::{error, info, warn};
use sp_core::{crypto::Pair as TraitPair, crypto::Ss58Codec};
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
    //TODO CHECK the timepoint, if exists, do not repeat
    let ctrl = AccountKeyring::Eve.to_account_id().into();
    let call = kusama::api::staking_bond_call::<KusamaRuntime>(
        &ctrl,
        MIN_POOL_BALANCE,
        staking::RewardDestination::Staked,
    );

    let account_id = AccountId::from_string(&pool_addr)
        .map_err(|e| Error::Other("parse pool_addr to account id error".to_string()))?;
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

/// If the wallet is the last one need to get 'TimePoint' and call 'as_multi'.
pub(crate) async fn do_last_relay_bond(
    others: Vec<AccountId>,
    pool_addr: String,
    subxt_client: &Client<KusamaRuntime>,
    signer: &(dyn Signer<KusamaRuntime> + Send + Sync),
) -> Result<(), Error> {
    info!("do_last_relay_bond");
    // let ctrl = AccountKeyring::Eve.to_account_id().into();
    // let call = kusama::api::staking_bond_call::<KusamaRuntime>(
    //     &ctrl,
    //     MIN_POOL_BALANCE,
    //     staking::RewardDestination::Staked,
    // );
    // let public =
    //     sp_core::ed25519::Public::from_str(pool_addr)
    //         .unwrap();

    // let kusama::api::MultisigData { when, .. } = subxt_client.fetch(&store, None).await?.unwrap();
    // println!("multisig timepoint{:?}", when);

    // let call_hash= kusama::api::multisig_call_hash(subxt_client, call).unwrap();

    // let when = get_time_point(subxt_client, public.into(), call_hash).await?;

    // // 3.1 approve the call and execute it
    // let bob = PairSigner::<RelayRuntime, _>::new(AccountKeyring::Bob.pair());
    // let mc = kusama::api::multisig_as_multi_call::<RelayRuntime, staking::BondCall<RelayRuntime>>(
    //     subxt_client,
    //     2,
    //     others,
    //     Some(Timepoint::new(when.height, when.index)),
    //     call,
    //     false,
    //     1_000_000_000_000,
    // )?;
    // let result = subxt_client.submit(mc, signer).await.unwrap();
    // println!("multisig_as_multi_call hash {:?}", result);
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
    //TODO CHECK the timepoint, if exists, do not repeat
    // 1.1 construct staking bond call
    let call = kusama::api::staking_bond_extra_call::<KusamaRuntime>(MIN_POOL_BALANCE);

    let account_id = AccountId::from_string(&pool_addr)
        .map_err(|e| Error::Other("parse pool_addr to account id error".to_string()))?;
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
    Ok(())
}
