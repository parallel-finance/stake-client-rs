use super::transaction;
use super::AccountId;
use super::Amount;
use super::HeikoRuntime;
use super::KusamaRuntime;
use super::TasksType;
use super::TASK_INTERVAL;
use crate::kusama::transaction::{
    do_relay_unbond, do_relay_withdraw_unbonded, do_xcm_transfer_to_para_chain,
};
use crate::primitives::RELAY_CHAIN_ERA_LOCKED;

use async_std::{sync::Arc, task};
use core::marker::PhantomData;
use log::{info, warn};
use runtime::kusama;
use sp_keyring::AccountKeyring;
use std::time;
use substrate_subxt::{Client, Signer};
use tokio::sync::{mpsc, oneshot};

pub async fn dispatch(
    relay_subxt_client: &Client<KusamaRuntime>,
    para_subxt_client: &Client<HeikoRuntime>,
    relay_signer: &(dyn Signer<KusamaRuntime> + Send + Sync),
    para_signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    mut system_rpc_rx: mpsc::Receiver<(TasksType, oneshot::Sender<u64>)>,
    others: Vec<AccountId>,
    relay_pool_addr: String,
    para_pool_addr: String,
    first: bool,
    mut withdraw_unbonded_amount: Arc<u128>,
) {
    // todo put this to database, because this will be lost when the client restart
    let mut unbonded_era_index_list: Vec<(AccountId, u32, Amount)> = vec![];
    loop {
        // try_next won't go on util finish this task
        match system_rpc_rx.recv().await {
            Some((task_type, response)) => match task_type {
                TasksType::RelayBond => {
                    info!("Start bond task");
                    relay_bond(
                        relay_subxt_client,
                        relay_signer,
                        others.clone(),
                        relay_pool_addr.clone(),
                        first,
                    )
                    .await;
                    response.send(0).unwrap();
                }
                TasksType::RelayBondExtra => {
                    info!("Start bond extra task");
                    relay_bond_extra(
                        relay_subxt_client,
                        relay_signer,
                        others.clone(),
                        relay_pool_addr.clone(),
                        first,
                    )
                    .await;
                    response.send(0).unwrap();
                }
                TasksType::ParaRecordRewards(amount) => {
                    info!("Start record rewards task");
                    para_record_rewards(
                        para_subxt_client,
                        para_signer,
                        others.clone(),
                        relay_pool_addr.clone(),
                        amount,
                        first,
                    )
                    .await;
                    response.send(0).unwrap();
                }
                TasksType::ParaRecordSlash(amount) => {
                    info!("Start record slash task");
                    para_record_slash(
                        para_subxt_client,
                        para_signer,
                        others.clone(),
                        relay_pool_addr.clone(),
                        amount,
                        first,
                    )
                    .await;
                }
                TasksType::ParaUnstake(_account_id, amount) => {
                    info!("Start unbond task");
                    start_unstake_task(&relay_subxt_client, amount.clone(), first.clone()).await;
                    response.send(0).unwrap();
                }
                TasksType::RelayUnbonded(_agent, amount) => {
                    info!("Found Unbonded event");
                    let store = kusama::api::CurrentEraStore::<KusamaRuntime> {
                        _runtime: PhantomData,
                    };
                    match relay_subxt_client.fetch(&store, None).await {
                        Ok(era) => {
                            if let Some(era_index) = era {
                                let ctrl = AccountKeyring::Eve.to_account_id().into();
                                info!("Record Unbonded era index:{:?}", era_index);
                                unbonded_era_index_list.push((ctrl, era_index, amount));
                            }
                        }
                        Err(e) => {
                            warn!("error fetch CurrentEraStore: {:?}", e);
                        }
                    }
                    response.send(0).unwrap();
                }
                TasksType::RelayEraIndexChanged(era_index) => {
                    info!("Start RelayEraIndexChanged task");

                    for (_ctr, era, amount) in unbonded_era_index_list.clone().into_iter() {
                        if era_index.clone() - era >= RELAY_CHAIN_ERA_LOCKED {
                            *Arc::make_mut(&mut withdraw_unbonded_amount) += amount;
                            info!("withdraw unbonded amount {:?}", withdraw_unbonded_amount);
                            let _ = do_relay_withdraw_unbonded(&relay_subxt_client, first)
                                .await
                                .map_err(|e| info!("error do_relay_withdraw_unbonded: {:?}", e));
                        }
                    }
                    response.send(0).unwrap();
                }
                TasksType::RelayWithdrawUnbonded(_agent, amount) => {
                    info!("Start RelayWithdrawUnbonded task");

                    let _ = do_xcm_transfer_to_para_chain(
                        &relay_subxt_client,
                        para_pool_addr.clone(),
                        amount.clone(),
                        first,
                    )
                    .await
                    .map_err(|e| info!("error do_xcm_transfer_to_para_chain: {:?}", e));

                    *Arc::make_mut(&mut withdraw_unbonded_amount) -= amount;
                    info!("withdraw unbonded amount {:?}", withdraw_unbonded_amount);
                }
            },
            None => info!("dispatch pending..."),
        }
        task::sleep(time::Duration::from_millis(TASK_INTERVAL)).await;
    }
}

async fn relay_bond(
    subxt_relay_client: &Client<KusamaRuntime>,
    relay_signer: &(dyn Signer<KusamaRuntime> + Send + Sync),
    others: Vec<AccountId>,
    pool_addr: String,
    first: bool,
) {
    info!("relay_bond");
    if first {
        let _ = transaction::do_first_relay_bond(
            others.clone(),
            pool_addr,
            &subxt_relay_client,
            relay_signer,
        )
        .await
        .map_err(|e| warn!("error do_first_relay_bond: {:?}", e));
    } else {
        task::sleep(time::Duration::from_millis(TASK_INTERVAL)).await;
        let _ = transaction::do_last_relay_bond(
            others.clone(),
            pool_addr,
            &subxt_relay_client,
            relay_signer,
        )
        .await
        .map_err(|e| warn!("error do_last_relay_bond: {:?}", e));
    }
}

async fn relay_bond_extra(
    subxt_relay_client: &Client<KusamaRuntime>,
    relay_signer: &(dyn Signer<KusamaRuntime> + Send + Sync),
    others: Vec<AccountId>,
    pool_addr: String,
    first: bool,
) {
    info!("relay_bond_extra");
    if first {
        let _ = transaction::do_first_relay_bond_extra(
            others.clone(),
            pool_addr,
            &subxt_relay_client,
            relay_signer,
        )
        .await
        .map_err(|e| warn!("error do_first_relay_bond_extra: {:?}", e));
    } else {
        task::sleep(time::Duration::from_millis(TASK_INTERVAL)).await;
        let _ = transaction::do_last_relay_bond_extra(
            others.clone(),
            pool_addr,
            &subxt_relay_client,
            relay_signer,
        )
        .await
        .map_err(|e| warn!("error do_last_relay_bond_extra: {:?}", e));
    }
}

async fn para_record_rewards(
    subxt_para_client: &Client<HeikoRuntime>,
    para_signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    others: Vec<AccountId>,
    pool_addr: String,
    amount: Amount,
    first: bool,
) {
    info!("para_record_rewards {:?}", amount);
    if first {
        let _ = transaction::do_first_para_record_rewards(
            others.clone(),
            pool_addr,
            &subxt_para_client,
            para_signer,
            amount,
        )
        .await
        .map_err(|e| warn!("error do_first_para_record_rewards: {:?}", e));
    } else {
        let _ = transaction::do_last_para_record_rewards(
            others.clone(),
            pool_addr,
            &subxt_para_client,
            para_signer,
            amount,
        )
        .await
        .map_err(|e| warn!("error do_last_para_record_rewards: {:?}", e));
    }
}

async fn para_record_slash(
    subxt_para_client: &Client<HeikoRuntime>,
    para_signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    others: Vec<AccountId>,
    pool_addr: String,
    amount: Amount,
    first: bool,
) {
    info!("para_record_slash {:?}", amount);
    if first {
        let _ = transaction::do_first_para_record_slash(
            others.clone(),
            pool_addr,
            &subxt_para_client,
            para_signer,
            amount,
        )
        .await
        .map_err(|e| warn!("error do_first_para_record_slash: {:?}", e));
    } else {
        let _ = transaction::do_last_para_record_slash(
            others.clone(),
            pool_addr,
            &subxt_para_client,
            para_signer,
            amount,
        )
        .await
        .map_err(|e| warn!("error do_last_para_record_slash: {:?}", e));
    }
}

/// start withdraw task, ws_server: ws://127.0.0.1:9944
pub(crate) async fn start_unstake_task(
    relay_subxt_client: &Client<KusamaRuntime>,
    amount: Amount,
    first: bool,
) {
    // controller do unbond, current controller is Eve
    // current controller is Eve
    if first {
        let _ = do_relay_unbond(&relay_subxt_client, amount.clone())
            .await
            .map_err(|e| warn!("error do_relay_unbond: {:?}", e));
    }
}
