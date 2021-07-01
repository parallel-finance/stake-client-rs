use crate::listener;
use crate::primitives::{AccountId, Amount, TasksType};
use crate::tasks::{
    do_first_finish_processed_unstake, do_first_process_pending_unstake, do_first_withdraw,
    do_last_finish_processed_unstake, do_last_process_pending_unstake, do_last_withdraw,
    wait_transfer_finished,
};

use async_std::task;
use core::marker::PhantomData;
use futures::join;
use parallel_primitives::{Balance, CurrencyId, PriceWithDecimal};
use runtime::error::Error;
use runtime::heiko::runtime::HeikoRuntime;
use runtime::kusama::{self, runtime::KusamaRuntime as RelayRuntime};
use sp_core::crypto::Ss58Codec;
use sp_core::Pair;
use sp_keyring::AccountKeyring;
use std::time;
use substrate_subxt::system::System;
use substrate_subxt::{Client, ClientBuilder, PairSigner};
use substrate_subxt::{Error as SubError, Signer};
use tokio::sync::{mpsc, oneshot};

const RELAY_CHAIN_BOND_SEED: &str = "//Eve";
const FROM_RELAY_CHAIN_SEED: &str = "//Alice";
const TO_REPLAY_CHAIN_ADDRESS: &str = "5DjYJStmdZ2rcqXbXGX7TW85JsrW6uG4y9MUcLq2BoPMpRA7";
const FROM_PARA_CHAIN_SEED: &str = "//Eve";
// todo change to relay chain Bonding Duration
const RELAY_CHAIN_ERA_LOCKED: u32 = 1;

pub async fn run(
    threshold: u16,
    pair: sp_core::sr25519::Pair,
    others: Vec<AccountId>,
    para_ws_server: &str,
    relay_ws_server: &str,
    pool_addr: &str,
    multi_addr: &str,
    currency_id: CurrencyId,
    first: bool,
) -> Result<(), Error> {
    // initialize heiko related api
    // todo register all unknown type
    let para_subxt_client = ClientBuilder::<HeikoRuntime>::new()
        .set_url(para_ws_server)
        .register_type_size::<CurrencyId>("CurrencyIdOf<T>")
        .register_type_size::<Balance>("BalanceOf<T>")
        .register_type_size::<<HeikoRuntime as System>::AccountId>("T::AccountId")
        .register_type_size::<CurrencyId>("T::CurrencyId")
        .register_type_size::<Balance>("T::Balance")
        .register_type_size::<CurrencyId>("T::OracleKey")
        .register_type_size::<PriceWithDecimal>("T::OracleValue")
        .skip_type_sizes_check()
        .build()
        .await
        .unwrap();

    // todo register all unknown type
    let relay_subxt_client = ClientBuilder::<RelayRuntime>::new()
        .set_url(relay_ws_server)
        .register_type_size::<<RelayRuntime as System>::AccountId>("T::AccountId")
        .skip_type_sizes_check()
        // .register_type_size::<([u8; 20])>("EthereumAddress")
        .build()
        .await
        .unwrap();

    let multi_account_id = AccountId::from_string(multi_addr).unwrap();
    let pool_account_id = AccountId::from_string(pool_addr).unwrap();
    let para_signer = PairSigner::<HeikoRuntime, sp_core::sr25519::Pair>::new(pair.clone());

    // initial channel
    let (system_rpc_tx, system_rpc_rx) = mpsc::channel::<(TasksType, oneshot::Sender<u64>)>(5);

    // initial multi threads to listen on-chain status
    let l = listener::listener(
        system_rpc_tx,
        &para_subxt_client,
        &relay_subxt_client,
        pool_account_id.clone(),
        currency_id.clone(),
    );

    // initial task to receive order and dive
    let t = dispatch(
        system_rpc_rx,
        &para_subxt_client,
        &relay_subxt_client,
        &para_signer,
        multi_account_id,
        pool_account_id,
        threshold,
        others,
        first,
    );
    join!(l, t);
    Ok(())
}

pub async fn dispatch(
    mut system_rpc_rx: mpsc::Receiver<(TasksType, oneshot::Sender<u64>)>,
    para_subxt_client: &Client<HeikoRuntime>,
    relay_subxt_client: &Client<RelayRuntime>,
    para_signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    multi_account_id: AccountId,
    pool_account_id: AccountId,
    threshold: u16,
    others: Vec<AccountId>,
    first: bool,
) {
    // todo put this to database, because this will be lost when the client restart
    let mut unstake_list: Vec<(AccountId, Amount)> = vec![];
    let mut unbonded_list: Vec<(AccountId, Amount)> = vec![];
    let mut unbonded_era_index_list: Vec<(AccountId, u32)> = vec![];
    loop {
        match system_rpc_rx.recv().await {
            Some((task_type, response)) => match task_type {
                TasksType::ParaStake(amount) => {
                    println!("[+] Start ParaStake task");
                    let _ = start_withdraw_task_para(
                        &para_subxt_client,
                        &relay_subxt_client,
                        para_signer,
                        multi_account_id.clone(),
                        threshold.clone(),
                        others.clone(),
                        amount.clone(),
                        first.clone(),
                    )
                    .await
                    .map_err(|e| println!("error start_withdraw_task_para: {:?}", e));
                    response.send(0).unwrap();
                }
                TasksType::ParaUnstake(owner, amount) => {
                    println!("[+] Start ParaUnstake task");
                    let _ = start_unstake_task(&relay_subxt_client, amount.clone(), first.clone())
                        .await
                        .map_err(|e| println!("error start_withdraw_task_para: {:?}", e));
                    response.send(0).unwrap();
                    unstake_list.push((owner, amount));
                }
                TasksType::RelayUnbonded(agent, amount) => {
                    println!("[+] Start RelayUnbonded task");
                    let mut index = 0;
                    for (owner, a) in unstake_list.clone().into_iter() {
                        if amount == a {
                            let store = kusama::api::CurrentEraStore::<RelayRuntime> {
                                _runtime: PhantomData,
                            };
                            match relay_subxt_client.fetch(&store, None).await {
                                Ok(era) => {
                                    if let Some(era_index) = era {
                                        let ctrl = AccountKeyring::Eve.to_account_id().into();
                                        unbonded_era_index_list.push((ctrl, era_index));
                                    }
                                }
                                Err(e) => {
                                    println!("error fetch CurrentEraStore: {:?}", e);
                                }
                            }

                            let _ = start_process_pending_unstake_task_para(
                                &para_subxt_client,
                                para_signer,
                                multi_account_id.clone(),
                                threshold.clone(),
                                others.clone(),
                                agent.clone(),
                                owner.clone(),
                                amount.clone(),
                                first.clone(),
                            )
                            .await
                            .map_err(|e| println!("error start_withdraw_task_para: {:?}", e));
                            response.send(0).unwrap();
                            unbonded_list.push((owner, amount));
                            break;
                        }
                        index = index + 1
                    }
                    unstake_list.remove(index);
                }
                TasksType::RelayWithdrawUnbonded(agent, mut amount) => {
                    println!("[+] Start RelayWithdrawUnbonded task");
                    let mut count = 0;
                    for (owner, a) in unbonded_list.clone().into_iter() {
                        if amount >= a {
                            let _ = start_finish_processed_unstake_task_para(
                                &para_subxt_client,
                                para_signer,
                                multi_account_id.clone(),
                                pool_account_id.clone(),
                                threshold.clone(),
                                others.clone(),
                                agent.clone(),
                                owner.clone(),
                                amount.clone(),
                                first.clone(),
                            )
                            .await
                            .map_err(|e| println!("error start_withdraw_task_para: {:?}", e));
                        }
                        amount = amount - a;
                        count = count + 1;
                    }
                    for i in (count - 1)..0 {
                        unbonded_list.remove(i);
                    }
                    response.send(0).unwrap();
                }
                TasksType::RelayEraIndexChanged(era_index) => {
                    println!("[+] Start RelayEraIndexChanged task");
                    for (_ctr, era) in unbonded_era_index_list.clone().into_iter() {
                        if era_index.clone() - era >= RELAY_CHAIN_ERA_LOCKED {
                            let _ = do_relay_withdraw_unbonded(&relay_subxt_client)
                                .await
                                .map_err(|e| println!("error do_relay_withdraw_unbonded: {:?}", e));
                        }
                    }
                }
            },
            None => println!("dispatch pending..."),
        }
    }
}

/// start withdraw task
pub(crate) async fn start_withdraw_task_para(
    para_subxt_client: &Client<HeikoRuntime>,
    relay_subxt_client: &Client<RelayRuntime>,
    para_signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    multi_account_id: AccountId,
    threshold: u16,
    others: Vec<AccountId>,
    amount: Amount,
    first: bool,
) -> Result<(), Error> {
    if first {
        let call_hash = do_first_withdraw(
            others.clone(),
            &para_subxt_client,
            para_signer,
            amount.clone(),
            threshold.clone(),
        )
        .await?;
        let _ =
            wait_transfer_finished(&para_subxt_client, multi_account_id.clone(), call_hash).await?;
        println!("[+] Create withdraw transaction finished");
    } else {
        let call_hash = do_last_withdraw(
            others.clone(),
            multi_account_id.clone(),
            &para_subxt_client,
            para_signer,
            amount.clone(),
            threshold.clone(),
        )
        .await?;
        let _ =
            wait_transfer_finished(&para_subxt_client, multi_account_id.clone(), call_hash).await?;
        println!("[+] Create withdraw transaction finished");

        // todo this is just mock: transfer relay chain amount from one address to other
        let _ = transfer_relay_chain_balance(&relay_subxt_client, amount.clone()).await?;
        println!("[+] Create mock relay transaction finished");
    }
    Ok(())
}

/// start process_pending_unstake task
pub(crate) async fn start_process_pending_unstake_task_para(
    para_subxt_client: &Client<HeikoRuntime>,
    para_signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    multi_account_id: AccountId,
    threshold: u16,
    others: Vec<AccountId>,
    agent: AccountId,
    owner: AccountId,
    amount: Amount,
    first: bool,
) -> Result<(), Error> {
    if first {
        let call_hash = do_first_process_pending_unstake(
            others.clone(),
            &para_subxt_client,
            para_signer,
            agent,
            owner,
            amount.clone(),
            threshold.clone(),
        )
        .await?;
        let _ =
            wait_transfer_finished(&para_subxt_client, multi_account_id.clone(), call_hash).await?;
        println!("[+] Create process pending unstake transaction finished");
    } else {
        let call_hash = do_last_process_pending_unstake(
            others.clone(),
            multi_account_id.clone(),
            &para_subxt_client,
            para_signer,
            agent,
            owner,
            amount.clone(),
            threshold.clone(),
        )
        .await?;
        let _ =
            wait_transfer_finished(&para_subxt_client, multi_account_id.clone(), call_hash).await?;
        println!("[+] Create process pending unstake transaction finished");
    }
    Ok(())
}

/// start finish_processed_unstake task
pub(crate) async fn start_finish_processed_unstake_task_para(
    para_subxt_client: &Client<HeikoRuntime>,
    para_signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    multi_account_id: AccountId,
    pool_account_id: AccountId,
    threshold: u16,
    others: Vec<AccountId>,
    agent: AccountId,
    owner: AccountId,
    amount: Amount,
    first: bool,
) -> Result<(), Error> {
    if first {
        // transfer from eve to multi-address first
        // todo move it out from here and use XCM
        let _ = transfer_para_from_eve_to_pool(&para_subxt_client, pool_account_id, amount.clone())
            .await?;

        let call_hash = do_first_finish_processed_unstake(
            others.clone(),
            &para_subxt_client,
            para_signer,
            agent,
            owner,
            amount.clone(),
            threshold.clone(),
        )
        .await?;
        let _ =
            wait_transfer_finished(&para_subxt_client, multi_account_id.clone(), call_hash).await?;
        println!("[+] Create finish processed unstake transaction finished");
    } else {
        let call_hash = do_last_finish_processed_unstake(
            others.clone(),
            multi_account_id.clone(),
            &para_subxt_client,
            para_signer,
            agent,
            owner,
            amount.clone(),
            threshold.clone(),
        )
        .await?;
        let _ =
            wait_transfer_finished(&para_subxt_client, multi_account_id.clone(), call_hash).await?;
        println!("[+] Create finish processed unstake transaction finished");
    }
    Ok(())
}

async fn transfer_para_from_eve_to_pool(
    subxt_client: &Client<HeikoRuntime>,
    pool_account_id: AccountId,
    amount: u128,
) -> Result<(), Error> {
    println!(
        "[+] Create para chain transaction from eve to pool, amount:{:?}",
        amount
    );
    let pair = sp_core::sr25519::Pair::from_string(&FROM_PARA_CHAIN_SEED, None)
        .map_err(|_err| SubError::Other("failed to create pair from seed".to_string()))?;
    let signer = PairSigner::<HeikoRuntime, sp_core::sr25519::Pair>::new(pair.clone());
    let pool: sp_runtime::MultiAddress<sp_runtime::AccountId32, u32> = pool_account_id.into();

    let call =
        kusama::api::currencies_transfer_call::<HeikoRuntime>(&pool, CurrencyId::KSM, amount);
    let result = subxt_client.submit(call, &signer).await.map_err(|e| {
        println!("{:?}", e);
        SubError::Other("failed to create transaction".to_string())
    })?;

    println!(
        "[+] transfer_para_from_eve_to_pool finished, para chain call hash {:?}",
        result
    );
    Ok(())
}

async fn transfer_relay_chain_balance(
    subxt_client: &Client<RelayRuntime>,
    amount: u128,
) -> Result<(), Error> {
    println!("[+] Create relay chain transaction");
    // let pair = sp_core::ed25519::Pair::from_string(&FROM_RELAY_CHAIN_SEED, None)
    //     .map_err(|_err| SubError::Other("failed to create pair from seed".to_string()))?;
    // let signer = PairSigner::<RelayRuntime, sp_core::ed25519::Pair>::new(pair.clone());
    // let to_account_id = AccountId::from_string(TO_REPLAY_CHAIN_ADDRESS)
    let pair = sp_core::sr25519::Pair::from_string(&FROM_RELAY_CHAIN_SEED, None)
        .map_err(|_err| SubError::Other("failed to create pair from seed".to_string()))?;
    let signer = PairSigner::<RelayRuntime, sp_core::sr25519::Pair>::new(pair.clone());
    let to_account_id = AccountId::from_string(TO_REPLAY_CHAIN_ADDRESS)
        .map_err(|_| SubError::Other("invalid replay address".to_string()))?;
    let ai: sp_runtime::MultiAddress<sp_runtime::AccountId32, u32> = to_account_id.into();

    let call = kusama::api::balances_transfer_call::<RelayRuntime>(&ai, amount);
    let result = subxt_client.submit(call, &signer).await.map_err(|e| {
        println!("{:?}", e);
        SubError::Other("failed to create transaction".to_string())
    })?;

    println!(
        "[+] transfer_relay_chain_balance finished, replay chain call hash {:?}",
        result
    );
    Ok(())
}

/// start withdraw task, ws_server: ws://127.0.0.1:9944
pub(crate) async fn start_unstake_task(
    relay_subxt_client: &Client<RelayRuntime>,
    amount: Amount,
    first: bool,
) -> Result<(), Error> {
    // controller do unbond, current controller is Eve
    if first {
        let _ = do_relay_unbond(&relay_subxt_client, amount.clone()).await?;

        // todo: remove me, it's just for mock
        task::sleep(time::Duration::from_secs(20)).await;
        let _ = do_relay_withdraw_unbonded(&relay_subxt_client).await?;
    } else {
        // current controller is Eve
    }
    Ok(())
}

async fn do_relay_unbond(subxt_client: &Client<RelayRuntime>, amount: u128) -> Result<(), Error> {
    println!("[+] Create relay chain unbond transaction");
    let pair = sp_core::sr25519::Pair::from_string(&RELAY_CHAIN_BOND_SEED, None)
        .map_err(|_err| SubError::Other("failed to create pair from seed".to_string()))?;
    let signer = PairSigner::<RelayRuntime, sp_core::sr25519::Pair>::new(pair.clone());
    let call = kusama::api::staking_unbond_call::<RelayRuntime>(amount);
    let result = subxt_client.submit(call, &signer).await.map_err(|e| {
        println!("{:?}", e);
        SubError::Other("failed to create unbond transaction".to_string())
    })?;

    println!(
        "[+] do_relay_unstake finished, replay chain call hash {:?}",
        result
    );
    Ok(())
}

async fn do_relay_withdraw_unbonded(subxt_client: &Client<RelayRuntime>) -> Result<(), Error> {
    println!("[+] Create relay chain withdraw unbonded transaction");
    let pair = sp_core::sr25519::Pair::from_string(&RELAY_CHAIN_BOND_SEED, None)
        .map_err(|_err| SubError::Other("failed to create pair from seed".to_string()))?;
    let signer = PairSigner::<RelayRuntime, sp_core::sr25519::Pair>::new(pair.clone());
    let call = kusama::api::staking_withdraw_unbonded_call::<RelayRuntime>(0);
    let result = subxt_client.submit(call, &signer).await.map_err(|e| {
        println!("{:?}", e);
        SubError::Other("failed to create withdraw unbonded transaction".to_string())
    })?;

    println!(
        "[+] do_relay_withdraw_unbonded finished, replay chain call hash {:?}",
        result
    );
    Ok(())
}
