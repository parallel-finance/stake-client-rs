use super::transaction;
use super::AccountId;
use super::Amount;
use super::HeikoRuntime;
use super::KusamaRuntime;
use super::TasksType;
use super::TASK_INTERVAL;
use async_std::task;
use log::{info, warn};
use sp_utils::mpsc::TracingUnboundedReceiver;
use std::time::Duration;
use substrate_subxt::{Client, Signer};
pub async fn dispatch(
    subxt_relay_client: &Client<KusamaRuntime>,
    subxt_para_client: &Client<HeikoRuntime>,
    relay_signer: &(dyn Signer<KusamaRuntime> + Send + Sync),
    para_signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    mut system_rpc_rx: TracingUnboundedReceiver<TasksType>,
    others: Vec<AccountId>,
    pool_addr: String,
    first: bool,
) {
    loop {
        // TODO: improve it for async, now this will process sequentially,
        // try_next won't go on util finish this task
        match system_rpc_rx.try_next() {
            Ok(some_tasks_type) => {
                if let Some(tasks_type) = some_tasks_type {
                    match tasks_type {
                        TasksType::RelayBond => {
                            relay_bond(
                                subxt_relay_client,
                                relay_signer,
                                others.clone(),
                                pool_addr.clone(),
                                first,
                            )
                            .await
                        }
                        TasksType::RelayBondExtra => {
                            relay_bond_extra(
                                subxt_relay_client,
                                relay_signer,
                                others.clone(),
                                pool_addr.clone(),
                                first,
                            )
                            .await
                        }
                        TasksType::ParaRecordRewards(amount) => {
                            para_record_rewards(subxt_para_client, para_signer, amount).await
                        }
                        TasksType::ParaRecordSlash(amount) => {
                            para_record_slash(subxt_para_client, para_signer, amount).await
                        }
                    }
                } else {
                    warn!("no task type");
                }
            }
            Err(e) => warn!("dispatch pending warn: {:?}", e),
        }
        task::sleep(Duration::from_millis(TASK_INTERVAL)).await;
        info!("waiting receive");
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
        task::sleep(Duration::from_millis(TASK_INTERVAL)).await;
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
        task::sleep(Duration::from_millis(TASK_INTERVAL)).await;
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
    _subxt_para_client: &Client<HeikoRuntime>,
    _para_signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    amount: Amount,
) {
    info!("para_record_rewards {:?}", amount);
}

async fn para_record_slash(
    _subxt_para_client: &Client<HeikoRuntime>,
    _para_signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    amount: Amount,
) {
    info!("para_record_slash {:?}", amount);
}
