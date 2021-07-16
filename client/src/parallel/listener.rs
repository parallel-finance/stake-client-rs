use crate::common::primitives::{AccountId, TasksType, MAX_WITHDRAW_BALANCE, MIN_WITHDRAW_BALANCE};
pub use parallel_primitives::CurrencyId;

use async_std::{
    sync::{Arc, Mutex},
    task,
};
use futures::join;
use runtime::heiko::{self, runtime::HeikoRuntime};
use runtime::kusama::runtime::KusamaRuntime as RelayRuntime;
use runtime::pallets::liquid_staking::UnstakedEvent;
use runtime::pallets::staking::{UnbondedEvent, WithdrawnEvent};
use sp_core::Decode;
use std::time;
use substrate_subxt::{Client, EventSubscription, RawEvent};
use tokio::sync::{mpsc, oneshot};

const LISTEN_INTERVAL: u64 = 5; // 5 sec
pub const LISTEN_WAIT_INTERVAL: u64 = 30; // 30 sec

pub async fn listener(
    system_rpc_tx: mpsc::Sender<(TasksType, oneshot::Sender<u64>)>,
    para_subxt_client: &Client<HeikoRuntime>,
    relay_subxt_client: &Client<RelayRuntime>,
    pool_account_id: AccountId,
    currency_id: CurrencyId,
    withdraw_unbonded_amount: Arc<Mutex<u128>>,
) {
    let l1 = listen_pool_balance(
        system_rpc_tx.clone(),
        para_subxt_client,
        pool_account_id.clone(),
        currency_id.clone(),
        withdraw_unbonded_amount.clone(),
    );
    let l2 = listen_unstaked_event(system_rpc_tx.clone(), para_subxt_client);
    let l3 = listen_unbonded_event(system_rpc_tx.clone(), relay_subxt_client);
    let l4 = listen_withdraw_unbonded_event(
        system_rpc_tx.clone(),
        relay_subxt_client,
        withdraw_unbonded_amount.clone(),
    );
    join!(l1, l2, l3, l4);
}

/// listen to the balance change of pool
pub(crate) async fn listen_pool_balance(
    system_rpc_tx: mpsc::Sender<(TasksType, oneshot::Sender<u64>)>,
    para_subxt_client: &Client<HeikoRuntime>,
    pool_account_id: AccountId,
    currency_id: CurrencyId,
    withdraw_unbonded_amount: Arc<Mutex<u128>>,
) {
    let store = heiko::api::AccountsStore::<HeikoRuntime> {
        account: pool_account_id,
        currency_id,
    };
    loop {
        match para_subxt_client.fetch(&store, None).await {
            Ok(r) => {
                if let Some(account_info) = r {
                    let balance = account_info.free - account_info.frozen;
                    if balance >= MIN_WITHDRAW_BALANCE + *withdraw_unbonded_amount.lock().await {
                        println!("[+] Pool's amount is {:?}， need to withdraw", balance);
                        let (resp_tx, resp_rx) = oneshot::channel();
                        let wa = *withdraw_unbonded_amount.lock().await;
                        if balance - wa < MAX_WITHDRAW_BALANCE {
                            system_rpc_tx
                                .clone()
                                .try_send((TasksType::ParaStake(balance - wa), resp_tx))
                                .ok();
                            let _res = resp_rx.await.ok();
                        } else {
                            system_rpc_tx
                                .clone()
                                .try_send((TasksType::ParaStake(MAX_WITHDRAW_BALANCE), resp_tx))
                                .ok();
                            let _res = resp_rx.await.ok();
                        }
                        task::sleep(time::Duration::from_secs(LISTEN_WAIT_INTERVAL)).await;
                    }
                }
            }
            Err(e) => {
                println!("listen_pool_balance error: {:?}", e);
            }
        }
        task::sleep(time::Duration::from_secs(LISTEN_INTERVAL)).await;
    }
}

/// listen to the unstaked event
async fn listen_unstaked_event(
    mut system_rpc_tx: mpsc::Sender<(TasksType, oneshot::Sender<u64>)>,
    para_subxt_client: &Client<HeikoRuntime>,
) {
    let sub = para_subxt_client
        .subscribe_finalized_events()
        .await
        .unwrap();
    let decoder = para_subxt_client.events_decoder();
    let mut sub = EventSubscription::<HeikoRuntime>::new(sub, &decoder);
    sub.filter_event::<UnstakedEvent<HeikoRuntime>>();
    loop {
        match sub
            .next()
            .await
            .and_then(|result_raw| -> Option<RawEvent> {
                println!("RawEvent:{:?}", result_raw);
                result_raw.ok()
            })
            .and_then(|raw| -> Option<UnstakedEvent<HeikoRuntime>> {
                UnstakedEvent::<HeikoRuntime>::decode(&mut &raw.data[..]).ok()
            }) {
            Some(event) => {
                println!("[+] Received Unstaked event: {:?}", &event);
                let (resp_tx, resp_rx) = oneshot::channel();
                system_rpc_tx
                    .try_send((TasksType::ParaUnstake(event.account, event.amount), resp_tx))
                    .ok();
                let _res = resp_rx.await.ok();
            }
            None => {}
        }
    }
}

/// listen to the unbonded event
async fn listen_unbonded_event(
    mut system_rpc_tx: mpsc::Sender<(TasksType, oneshot::Sender<u64>)>,
    relay_subxt_client: &Client<RelayRuntime>,
) {
    let sub = relay_subxt_client
        .subscribe_finalized_events()
        .await
        .unwrap();
    let decoder = relay_subxt_client.events_decoder();
    let mut sub = EventSubscription::<RelayRuntime>::new(sub, &decoder);
    sub.filter_event::<UnbondedEvent<RelayRuntime>>();
    loop {
        match sub
            .next()
            .await
            .and_then(|result_raw| -> Option<RawEvent> {
                println!("RawEvent:{:?}", result_raw);
                result_raw.ok()
            })
            .and_then(|raw| -> Option<UnbondedEvent<RelayRuntime>> {
                UnbondedEvent::<RelayRuntime>::decode(&mut &raw.data[..]).ok()
            }) {
            Some(event) => {
                println!("[+] Received Unbonded event: {:?}", &event);
                let (resp_tx, resp_rx) = oneshot::channel();
                system_rpc_tx
                    .try_send((
                        TasksType::RelayUnbonded(event.account, event.amount),
                        resp_tx,
                    ))
                    .ok();
                let _res = resp_rx.await.ok();
            }
            None => {}
        }
    }
}

/// listen to the withdraw unbonded event
async fn listen_withdraw_unbonded_event(
    mut system_rpc_tx: mpsc::Sender<(TasksType, oneshot::Sender<u64>)>,
    relay_subxt_client: &Client<RelayRuntime>,
    withdraw_unbonded_amount: Arc<Mutex<u128>>,
) {
    let sub = relay_subxt_client
        .subscribe_finalized_events()
        .await
        .unwrap();
    let decoder = relay_subxt_client.events_decoder();
    let mut sub = EventSubscription::<RelayRuntime>::new(sub, &decoder);
    sub.filter_event::<WithdrawnEvent<RelayRuntime>>();
    loop {
        match sub
            .next()
            .await
            .and_then(|result_raw| -> Option<RawEvent> {
                println!("RawEvent:{:?}", result_raw);
                result_raw.ok()
            })
            .and_then(|raw| -> Option<WithdrawnEvent<RelayRuntime>> {
                WithdrawnEvent::<RelayRuntime>::decode(&mut &raw.data[..]).ok()
            }) {
            Some(event) => {
                println!("[+] Received Withdrawn event: {:?}", &event);
                let (resp_tx, resp_rx) = oneshot::channel();
                *withdraw_unbonded_amount.lock().await += event.amount;

                //todo need to wait until asset has been transfered to pool address.
                // task::sleep(time::Duration::from_secs(LISTEN_WAIT_INTERVAL)).await;

                system_rpc_tx
                    .try_send((
                        TasksType::RelayWithdrawUnbonded(event.account, event.amount),
                        resp_tx,
                    ))
                    .ok();
                let _res = resp_rx.await.ok();
            }
            None => {}
        }
    }
}
