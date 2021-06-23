use crate::primitives::{AccountId, TasksType, MAX_WITHDRAW_BALANCE, MIN_WITHDRAW_BALANCE};
use async_std::task;
use futures::join;
pub use parallel_primitives::CurrencyId;
use runtime::heiko;
use runtime::heiko::runtime::HeikoRuntime;
use runtime::pallets::liquid_staking::UnstakedEvent;
use sp_core::Decode;
use std::time;
use substrate_subxt::{Client, EventSubscription, RawEvent};
use tokio::sync::{mpsc, oneshot};

const LISTEN_INTERVAL: u64 = 5; // 5 sec
pub async fn listener(
    system_rpc_tx: mpsc::Sender<(TasksType, oneshot::Sender<u64>)>,
    para_subxt_client: &Client<HeikoRuntime>,
    pool_account_id: AccountId,
    currency_id: CurrencyId,
) {
    let l1 = listen_pool_balances(
        system_rpc_tx.clone(),
        para_subxt_client,
        pool_account_id.clone(),
        currency_id.clone(),
    );
    let l2 = listen_unstake_event(system_rpc_tx, para_subxt_client);
    join!(l1, l2);
}

/// listen to the balance change of pool
pub(crate) async fn listen_pool_balances(
    system_rpc_tx: mpsc::Sender<(TasksType, oneshot::Sender<u64>)>,
    para_subxt_client: &Client<HeikoRuntime>,
    pool_account_id: AccountId,
    currency_id: CurrencyId,
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
                    if balance >= MIN_WITHDRAW_BALANCE {
                        println!("[+] Pool's amount is {:?}， need to withdraw", balance);
                        let (resp_tx, resp_rx) = oneshot::channel();
                        if balance < MAX_WITHDRAW_BALANCE {
                            system_rpc_tx
                                .clone()
                                .try_send((TasksType::ParaStake(balance), resp_tx))
                                .ok();
                            let _res = resp_rx.await.ok();
                        } else {
                            system_rpc_tx
                                .clone()
                                .try_send((TasksType::ParaStake(MAX_WITHDRAW_BALANCE), resp_tx))
                                .ok();
                            let _res = resp_rx.await.ok();
                        }
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

/// listen to the balance change of pool
async fn listen_unstake_event(
    mut system_rpc_tx: mpsc::Sender<(TasksType, oneshot::Sender<u64>)>,
    para_subxt_client: &Client<HeikoRuntime>,
) {
    let sub = para_subxt_client
        .subscribe_finalized_events()
        .await
        .unwrap();
    let decoder = para_subxt_client.events_decoder();
    let mut sub = EventSubscription::<HeikoRuntime>::new(sub, &decoder);
    sub.filter_event::<UnstakedEvent<_>>();
    loop {
        let (resp_tx, resp_rx) = oneshot::channel();
        let _ = sub
            .next()
            .await
            .and_then(|result_raw| -> Option<RawEvent> { result_raw.ok() })
            .and_then(|raw| -> Option<UnstakedEvent<HeikoRuntime>> {
                UnstakedEvent::<HeikoRuntime>::decode(&mut &raw.data[..]).ok()
            })
            .and_then(|event| -> Option<()> {
                println!("[+] Received Unstaked event: {:?}", &event);
                system_rpc_tx
                    .try_send((TasksType::ParaUnstake(event.amount), resp_tx))
                    .ok()
            });
        let _res = resp_rx.await.ok();
    }
}
