use super::kusama;
use super::KusamaRuntime;
use super::TasksType;
use super::MIN_BOND_BALANCE;
use super::{LISTEN_INTERVAL, TASK_INTERVAL};

use async_std::task;
use core::marker::PhantomData;
use futures::join;
use log::{debug, error, info};
use runtime::heiko::runtime::HeikoRuntime;
use runtime::pallets::liquid_staking::UnstakedEvent;
use runtime::pallets::staking::{RewardEvent, SlashEvent, UnbondedEvent};
use sp_core::Decode;
use std::str::FromStr;
use std::time::Duration;
use substrate_subxt::{system::System, Client, EventSubscription, RawEvent};
use tokio::sync::{mpsc, oneshot};

pub async fn listener(
    relay_subxt_client: &Client<KusamaRuntime>,
    para_subxt_client: &Client<HeikoRuntime>,
    system_rpc_tx: mpsc::Sender<(TasksType, oneshot::Sender<u64>)>,
    pool_addr: String,
) {
    // start future-1 listening relaychain multisig-account balance
    let l1 = listen_agent_balance(
        relay_subxt_client.clone(),
        system_rpc_tx.clone(),
        pool_addr.clone(),
    );
    // start future-2 listening relaychain slash&reward
    let l2 = listen_reward(relay_subxt_client.clone(), system_rpc_tx.clone());
    let l3 = listen_slash(relay_subxt_client.clone(), system_rpc_tx.clone());
    let l4 = listen_unstaked_event(system_rpc_tx.clone(), para_subxt_client);
    let l5 = listen_unbonded_event(system_rpc_tx.clone(), relay_subxt_client);
    let l6 = listen_relay_chain_era(system_rpc_tx.clone(), relay_subxt_client);

    info!("listener join");
    join!(l1, l2, l3, l4, l5, l6);
}

async fn listen_agent_balance(
    subxt_relay_client: Client<KusamaRuntime>,
    system_rpc_tx: mpsc::Sender<(TasksType, oneshot::Sender<u64>)>,
    pool_addr: String,
) {
    let account_id: <KusamaRuntime as System>::AccountId =
        sp_core::ed25519::Public::from_str(&pool_addr)
            .unwrap()
            .into();
    let account = kusama::api::AccountStore::<KusamaRuntime> {
        account: account_id.clone(),
    };
    let bond = kusama::api::BondedStore::<KusamaRuntime> {
        stash: account_id.clone(),
    };

    info!("loop listen balance");
    loop {
        match subxt_relay_client.fetch(&account, None).await {
            Ok(account_store) => {
                info!(
                    "account id: {:?}, account_store: {:?}",
                    &account_id, &account_store
                );
                let bond_controller: Option<<KusamaRuntime as System>::AccountId> =
                    subxt_relay_client.fetch(&bond, None).await.unwrap();
                info!("bond_controller: {:?}", &bond_controller);
                let (resp_tx, resp_rx) = oneshot::channel();
                let r = account_store.and_then(|account_store| -> Option<()> {
                    let free = account_store.data.free;
                    let misc_frozen = account_store.data.misc_frozen;
                    //for now, make the loop interval longer.
                    if free - misc_frozen >= MIN_BOND_BALANCE {
                        match bond_controller {
                            Some(_bond) => {
                                system_rpc_tx
                                    .clone()
                                    .try_send((TasksType::RelayBondExtra, resp_tx))
                                    .ok();
                            }
                            None => {
                                system_rpc_tx
                                    .clone()
                                    .try_send((TasksType::RelayBond, resp_tx))
                                    .ok();
                            }
                        }
                    }
                    Some(())
                });
                let _res = resp_rx.await.ok();
                debug!("listen_balance option: {:?}", r);
            }
            Err(e) => {
                error!("listen_balance error: {:?}", e);
            }
        }

        task::sleep(Duration::from_millis(LISTEN_INTERVAL)).await;
    }
}

async fn listen_reward(
    subxt_relay_client: Client<KusamaRuntime>,
    mut system_rpc_tx: mpsc::Sender<(TasksType, oneshot::Sender<u64>)>,
) {
    let sub = subxt_relay_client
        .subscribe_finalized_events()
        .await
        .unwrap();
    let decoder = subxt_relay_client.events_decoder();
    let mut sub = EventSubscription::<KusamaRuntime>::new(sub, &decoder);
    sub.filter_event::<RewardEvent<_>>();
    loop {
        info!("loop listen_reward");
        match sub
            .next()
            .await
            .and_then(|result_raw| -> Option<RawEvent> { result_raw.ok() })
            .and_then(|raw| -> Option<RewardEvent<KusamaRuntime>> {
                RewardEvent::<KusamaRuntime>::decode(&mut &raw.data[..]).ok()
            }) {
            Some(event) => {
                info!("Receive Event: {:?}", &event);
                let (resp_tx, resp_rx) = oneshot::channel();
                system_rpc_tx
                    .try_send((TasksType::ParaRecordRewards(event.amount), resp_tx))
                    .ok();
                let _res = resp_rx.await.ok();
                info!("Record reword event finished");
            }
            None => {}
        }
    }
}

async fn listen_slash(
    subxt_relay_client: Client<KusamaRuntime>,
    mut system_rpc_tx: mpsc::Sender<(TasksType, oneshot::Sender<u64>)>,
) {
    let sub = subxt_relay_client
        .subscribe_finalized_events()
        .await
        .unwrap();
    let decoder = subxt_relay_client.events_decoder();
    let mut sub = EventSubscription::<KusamaRuntime>::new(sub, &decoder);
    sub.filter_event::<SlashEvent<_>>();
    loop {
        info!("loop listen_slash");
        match sub
            .next()
            .await
            .and_then(|result_raw| -> Option<RawEvent> { result_raw.ok() })
            .and_then(|raw| -> Option<SlashEvent<KusamaRuntime>> {
                SlashEvent::<KusamaRuntime>::decode(&mut &raw.data[..]).ok()
            }) {
            Some(event) => {
                info!("Receive Event: {:?}", &event);
                let (resp_tx, resp_rx) = oneshot::channel();
                system_rpc_tx
                    .try_send((TasksType::ParaRecordSlash(event.amount), resp_tx))
                    .ok();
                let _res = resp_rx.await.ok();
                info!("Record slash event finished");
            }
            None => {}
        }
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
        info!("loop listen unstaked event");
        match sub
            .next()
            .await
            .and_then(|result_raw| -> Option<RawEvent> {
                result_raw.ok()
            })
            .and_then(|raw| -> Option<UnstakedEvent<HeikoRuntime>> {
                UnstakedEvent::<HeikoRuntime>::decode(&mut &raw.data[..]).ok()
            }) {
            Some(event) => {
                info!("Received Unstaked event: {:?}", &event);
                let (resp_tx, resp_rx) = oneshot::channel();
                system_rpc_tx
                    .try_send((TasksType::ParaUnstake(event.account, event.amount), resp_tx))
                    .ok();
                let _res = resp_rx.await.ok();

                info!("Unstaked event processed");
            }
            None => {}
        }
    }
}

/// listen to the unbonded event
async fn listen_unbonded_event(
    mut system_rpc_tx: mpsc::Sender<(TasksType, oneshot::Sender<u64>)>,
    relay_subxt_client: &Client<KusamaRuntime>,
) {
    let sub = relay_subxt_client
        .subscribe_finalized_events()
        .await
        .unwrap();
    let decoder = relay_subxt_client.events_decoder();
    let mut sub = EventSubscription::<KusamaRuntime>::new(sub, &decoder);
    sub.filter_event::<UnbondedEvent<KusamaRuntime>>();
    loop {
        info!("loop listen unbonded event");
        match sub
            .next()
            .await
            .and_then(|result_raw| -> Option<RawEvent> {
                result_raw.ok()
            })
            .and_then(|raw| -> Option<UnbondedEvent<KusamaRuntime>> {
                UnbondedEvent::<KusamaRuntime>::decode(&mut &raw.data[..]).ok()
            }) {
            Some(event) => {
                info!("Received Unbonded event: {:?}", &event);
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
async fn listen_relay_chain_era(
    mut system_rpc_tx: mpsc::Sender<(TasksType, oneshot::Sender<u64>)>,
    relay_subxt_client: &Client<KusamaRuntime>,
) {
    let mut current_era_index: u32 = 0;
    info!("loop listen relay chain era");
    loop {
        let store = kusama::api::CurrentEraStore::<KusamaRuntime> {
            _runtime: PhantomData,
        };
        match relay_subxt_client.fetch(&store, None).await {
            Ok(era) => {
                if let Some(era_index) = era {
                    if era_index != current_era_index {
                        current_era_index = era_index;
                        let (resp_tx, resp_rx) = oneshot::channel();
                        system_rpc_tx
                            .try_send((
                                TasksType::RelayEraIndexChanged(current_era_index.clone()),
                                resp_tx,
                            ))
                            .ok();
                        let _res = resp_rx.await.ok();
                        info!("Current EraIndex changed {:?}", current_era_index);
                    }
                }
            }
            Err(e) => {
                info!("error fetch CurrentEraStore: {:?}", e);
            }
        }
        task::sleep(Duration::from_millis(TASK_INTERVAL)).await;
    }
}
