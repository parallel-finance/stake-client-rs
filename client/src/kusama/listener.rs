use super::kusama;
use super::KusamaRuntime;
use super::TasksType;
use super::LISTEN_INTERVAL;
use super::MIN_POOL_BALANCE;
use async_std::task;
use futures::join;
use log::{debug, error, info};
use sp_utils::mpsc::TracingUnboundedSender;
use std::str::FromStr;
use std::time::Duration;
use substrate_subxt::{system::System, Client};

pub async fn listener(
    subxt_relay_client: &Client<KusamaRuntime>,
    system_rpc_tx: TracingUnboundedSender<TasksType>,
    pool_addr: String,
) {
    // start future-1 listening relaychain multisig-account balance
    let f1 = listen_balance(
        subxt_relay_client.clone(),
        system_rpc_tx.clone(),
        pool_addr.clone(),
    );
    // start future-2 listening relaychain slash&reward
    let f2 = listen_slash_and_reward(subxt_relay_client.clone(), system_rpc_tx.clone(), pool_addr);
    info!("listener join");
    join!(f1, f2);
}

async fn listen_balance(
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
        let bond_controller: Option<<KusamaRuntime as System>::AccountId> =
            subxt_relay_client.fetch(&bond, None).await.unwrap();
        info!("bond_controller: {:?}", &bond_controller);
        match subxt_relay_client.fetch(&account, None).await {
            Ok(account_store) => {
                info!(
                    "account id: {:?}, account_store: {:?}",
                    &account_id, &account_store
                );
                let r = account_store.and_then(|account_store| -> Option<()> {
                    let free = account_store.data.free;
                    let misc_frozen = account_store.data.misc_frozen;
                    if free - misc_frozen >= MIN_POOL_BALANCE {
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
        info!("loop listen balance");
        task::sleep(Duration::from_millis(LISTEN_INTERVAL)).await;
    }
}

async fn listen_slash_and_reward(
    _subxt_relay_client: Client<KusamaRuntime>,
    mut system_rpc_tx: TracingUnboundedSender<TasksType>,
    _pool_addr: String,
) {
    //TODO 获取中继链链上状态,向平行链推送reward或slash
    loop {
        let _ = system_rpc_tx.start_send(TasksType::ParaRecordRewards);
        let _ = system_rpc_tx.start_send(TasksType::ParaRecordSlash);
        info!("loop listen_slash_and_reward");
        task::sleep(Duration::from_millis(LISTEN_INTERVAL * 6)).await;
    }
}
