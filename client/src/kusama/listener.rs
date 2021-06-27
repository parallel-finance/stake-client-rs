use super::kusama;
use super::KusamaRuntime;
use super::TasksType;
use super::LISTEN_INTERVAL;
use super::MIN_BOND_BALANCE;
use async_std::task;
use futures::join;
use log::{debug, error, info};
use runtime::pallets::staking::{RewardEvent, SlashEvent};
use sp_core::Decode;
use sp_utils::mpsc::TracingUnboundedSender;
use std::str::FromStr;
use std::time::Duration;
use substrate_subxt::{system::System, Client, EventSubscription, RawEvent};
pub async fn listener(
    subxt_relay_client: &Client<KusamaRuntime>,
    system_rpc_tx: TracingUnboundedSender<TasksType>,
    pool_addr: String,
) {
    // start future-1 listening relaychain multisig-account balance
    let f1 = listen_agent_balance(
        subxt_relay_client.clone(),
        system_rpc_tx.clone(),
        pool_addr.clone(),
    );
    // start future-2 listening relaychain slash&reward
    let f2 = listen_reward(subxt_relay_client.clone(), system_rpc_tx.clone());
    let f3 = listen_slash(subxt_relay_client.clone(), system_rpc_tx.clone());
    info!("listener join");
    join!(f1, f2, f3);
}

async fn listen_agent_balance(
    subxt_relay_client: Client<KusamaRuntime>,
    system_rpc_tx: TracingUnboundedSender<TasksType>,
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

    loop {
        info!("loop listen balance");
        match subxt_relay_client.fetch(&account, None).await {
            Ok(account_store) => {
                info!(
                    "account id: {:?}, account_store: {:?}",
                    &account_id, &account_store
                );
                let bond_controller: Option<<KusamaRuntime as System>::AccountId> =
                    subxt_relay_client.fetch(&bond, None).await.unwrap();
                info!("bond_controller: {:?}", &bond_controller);
                let r = account_store.and_then(|account_store| -> Option<()> {
                    let free = account_store.data.free;
                    let misc_frozen = account_store.data.misc_frozen;
                    //FIXME: bug, while do the last-mulsig about first-round, the second-round fisrt-mulsig is going.
                    //for now, make the loop interval longer.
                    if free - misc_frozen >= MIN_BOND_BALANCE {
                        let _ = bond_controller.map_or_else(
                            || system_rpc_tx.clone().start_send(TasksType::RelayBond),
                            |_bond_controller| {
                                system_rpc_tx.clone().start_send(TasksType::RelayBondExtra)
                            },
                        );
                    }
                    Some(())
                });
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
    mut system_rpc_tx: TracingUnboundedSender<TasksType>,
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
        let _ = sub
            .next()
            .await
            .and_then(|result_raw| -> Option<RawEvent> { result_raw.ok() })
            .and_then(|raw| -> Option<RewardEvent<KusamaRuntime>> {
                RewardEvent::<KusamaRuntime>::decode(&mut &raw.data[..]).ok()
            })
            .and_then(|event| -> Option<()> {
                info!("Receive Event: {:?}", &event);
                system_rpc_tx
                    .start_send(TasksType::ParaRecordRewards(event.amount))
                    .ok()
            });
    }
}

async fn listen_slash(
    subxt_relay_client: Client<KusamaRuntime>,
    mut system_rpc_tx: TracingUnboundedSender<TasksType>,
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
        let _ = sub
            .next()
            .await
            .and_then(|result_raw| -> Option<RawEvent> { result_raw.ok() })
            .and_then(|raw| -> Option<SlashEvent<KusamaRuntime>> {
                SlashEvent::<KusamaRuntime>::decode(&mut &raw.data[..]).ok()
            })
            .and_then(|event| -> Option<()> {
                info!("Receive Event: {:?}", &event);
                system_rpc_tx
                    .start_send(TasksType::ParaRecordSlash(event.amount))
                    .ok()
            });
    }
}
