use crate::listener;
use crate::primitives::{AccountId, Amount, TasksType, FOR_MOCK_SEED};
use crate::tasks::{
    do_first_finish_processed_unstake, do_first_process_pending_unstake, do_first_withdraw,
    do_last_finish_processed_unstake, do_last_process_pending_unstake, do_last_withdraw,
    wait_transfer_finished,
};

use async_std::sync::{Arc, Mutex};
use core::marker::PhantomData;
use frame_support::PalletId;
use futures::join;
use parallel_primitives::{Balance, CurrencyId, PriceWithDecimal};
use runtime::error::Error;
use runtime::heiko::runtime::HeikoRuntime;
use runtime::kusama::{self, runtime::KusamaRuntime as RelayRuntime};
use sp_core::crypto::Ss58Codec;
use sp_core::Pair;
use substrate_subxt::staking::Staking;
use substrate_subxt::system::System;
use substrate_subxt::{Client, ClientBuilder, PairSigner};
use substrate_subxt::{Error as SubError, Signer};
use tokio::sync::{mpsc, oneshot};
use xcm::v0::{MultiLocation, Outcome};

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
        .register_type_size::<CurrencyId>("CurrencyId")
        .register_type_size::<PalletId>("ParaId")
        .register_type_size::<MultiLocation>("MultiLocation")
        .register_type_size::<Outcome>("xcm::v0::Outcome")
        .register_type_size::<Outcome>("Outcome")
        .register_type_size::<([u8; 4], u64)>("MessageId")
        .skip_type_sizes_check()
        .build()
        .await
        .unwrap();

    // todo register all unknown type
    let relay_subxt_client = ClientBuilder::<RelayRuntime>::new()
        .set_url(relay_ws_server)
        .register_type_size::<<RelayRuntime as System>::AccountId>("T::AccountId")
        .register_type_size::<<RelayRuntime as Staking>::CandidateReceipt>("CandidateReceipt<Hash>")
        .register_type_size::<u32>("CoreIndex")
        .register_type_size::<u32>("GroupIndex")
        .register_type_size::<PalletId>("ParaId")
        .register_type_size::<MultiLocation>("MultiLocation")
        .register_type_size::<Outcome>("xcm::v0::Outcome")
        .register_type_size::<Outcome>("Outcome")
        .register_type_size::<([u8; 4], u64)>("MessageId")
        .skip_type_sizes_check()
        // .register_type_size::<([u8; 20])>("EthereumAddress")
        .build()
        .await
        .unwrap();

    let multi_account_id = AccountId::from_string(multi_addr).unwrap();
    let pool_account_id = AccountId::from_string(pool_addr).unwrap();
    let para_signer = PairSigner::<HeikoRuntime, sp_core::sr25519::Pair>::new(pair.clone());

    // initial channel
    let (system_rpc_tx, system_rpc_rx) = mpsc::channel::<(TasksType, oneshot::Sender<u64>)>(10);

    // todo put this to database, because this will be lost when the client restart
    let withdraw_unbonded_amount = Arc::new(Mutex::new(0));

    // initial multi threads to listen on-chain status
    let l = listener::listener(
        system_rpc_tx,
        &para_subxt_client,
        &relay_subxt_client,
        pool_account_id.clone(),
        currency_id.clone(),
        withdraw_unbonded_amount.clone(),
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
        withdraw_unbonded_amount.clone(),
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
    mut withdraw_unbonded_amount: Arc<Mutex<u128>>,
) {
    // todo put this to database, because this will be lost when the client restart
    let mut unstake_list: Vec<(AccountId, Amount)> = vec![];
    let mut unbonded_list: Vec<(AccountId, Amount)> = vec![];
    loop {
        match system_rpc_rx.recv().await {
            Some((task_type, response)) => match task_type {
                TasksType::ParaStake(amount) => {
                    println!("[+] Start withdraw task");
                    let _ = start_withdraw_task_para(
                        &para_subxt_client,
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
                    unstake_list.push((owner, amount));
                    response.send(0).unwrap();
                }
                TasksType::RelayUnbonded(agent, amount) => {
                    println!("[+] Start process pending unstake task");
                    let mut index = 0;
                    let mut found = false;
                    for (owner, a) in unstake_list.clone().into_iter() {
                        if amount == a {
                            found = true;

                            // get era_index
                            match get_era_index(relay_subxt_client).await {
                                Ok(era) => {
                                    let _ = start_process_pending_unstake_task_para(
                                        &para_subxt_client,
                                        para_signer,
                                        multi_account_id.clone(),
                                        threshold.clone(),
                                        others.clone(),
                                        agent.clone(),
                                        owner.clone(),
                                        era.clone(),
                                        amount.clone(),
                                        first.clone(),
                                    )
                                    .await
                                    .map_err(|e| {
                                        println!("process pending unstake task error: {:?}", e)
                                    });
                                    unbonded_list.push((owner, amount));
                                    break;
                                }
                                Err(e) => {
                                    println!("fetch CurrentEraStore error : {:?}", e);
                                    break;
                                }
                            }
                        }
                        index = index + 1
                    }
                    if found {
                        unstake_list.remove(index);
                    }
                    response.send(0).unwrap();
                }
                TasksType::RelayWithdrawUnbonded(agent, mut amount) => {
                    println!("[+] Start finish processed unstake task");
                    let mut count = 0;
                    for (owner, a) in unbonded_list.clone().into_iter() {
                        if amount < a {
                            break;
                        }

                        match start_finish_processed_unstake_task_para(
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
                        {
                            Ok(_result) => {
                                println!(" finish processed unstake succeed");
                                println!("#### amount:{} a:{}", amount, a);
                                amount -= a;
                                count = count + 1;
                                println!(
                                    "########## withdraw_unbonded_amount:{:?} - a:{:?}",
                                    withdraw_unbonded_amount, a
                                );
                                *withdraw_unbonded_amount.lock().await -= a;
                                println!(
                                    "########## after - withdraw_unbonded_amount:{:?}",
                                    withdraw_unbonded_amount
                                );
                            }
                            Err(e) => {
                                println!("finish processed unstake task error: {:?}", e);
                                break;
                            }
                        };
                    }
                    if count != 0 {
                        for i in (count - 1)..0 {
                            unbonded_list.remove(i);
                        }
                    }
                    response.send(0).unwrap();
                }
            },
            None => println!("dispatch pending..."),
        }
    }
}

async fn get_era_index(relay_subxt_client: &Client<RelayRuntime>) -> Result<u32, Error> {
    let store = kusama::api::CurrentEraStore::<RelayRuntime> {
        _runtime: PhantomData,
    };
    match relay_subxt_client.fetch(&store, None).await {
        Ok(era) => {
            if let Some(era_index) = era {
                Ok(era_index)
            } else {
                Err(Error::SubxtError(SubError::Other(
                    "invalid era index".to_string(),
                )))
            }
        }
        Err(e) => Err(Error::SubxtError(SubError::Other(
            "failed to fetch era index".to_string(),
        ))),
    }
}

/// start withdraw task
pub(crate) async fn start_withdraw_task_para(
    para_subxt_client: &Client<HeikoRuntime>,
    para_signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    multi_account_id: AccountId,
    threshold: u16,
    others: Vec<AccountId>,
    amount: Amount,
    first: bool,
) -> Result<(), Error> {
    if first {
        let _call_hash = do_first_withdraw(
            others.clone(),
            &para_subxt_client,
            para_signer,
            multi_account_id.clone(),
            amount.clone(),
            threshold.clone(),
        )
        .await?;
        println!("[+] Create withdraw transaction finished");
    } else {
        let _call_hash = do_last_withdraw(
            others.clone(),
            multi_account_id.clone(),
            &para_subxt_client,
            para_signer,
            amount.clone(),
            threshold.clone(),
        )
        .await?;
        println!("[+] Create withdraw transaction finished");
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
    era_index: u32,
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
            era_index.clone(),
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
            era_index.clone(),
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
    _pool_account_id: AccountId,
    threshold: u16,
    others: Vec<AccountId>,
    agent: AccountId,
    owner: AccountId,
    amount: Amount,
    first: bool,
) -> Result<(), Error> {
    if first {
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
    let pair = sp_core::sr25519::Pair::from_string(&FOR_MOCK_SEED, None)
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
