use crate::listener;
use crate::primitives::{AccountId, Amount, TasksType};
use crate::tasks::{do_first_withdraw, do_last_withdraw, wait_transfer_finished};

use futures::join;
use parallel_primitives::{Balance, CurrencyId};
use runtime::error::Error;
use runtime::heiko::runtime::HeikoRuntime;
use runtime::kusama::{self, runtime::KusamaRuntime as RelayRuntime};
use sp_core::crypto::Ss58Codec;
use sp_core::Pair;
use substrate_subxt::system::System;
use substrate_subxt::{Client, ClientBuilder, PairSigner};
use substrate_subxt::{Error as SubError, Signer};
use tokio::sync::{mpsc, oneshot};

const FROM_RELAY_CHAIN_SEED: &str = "//Alice";
const TO_REPLAY_CHAIN_ADDRESS: &str = "5DjYJStmdZ2rcqXbXGX7TW85JsrW6uG4y9MUcLq2BoPMpRA7";

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
    let para_subxt_client = ClientBuilder::<HeikoRuntime>::new()
        .set_url(para_ws_server)
        .register_type_size::<<HeikoRuntime as System>::AccountId>("T::AccountId")
        .register_type_size::<CurrencyId>("CurrencyIdOf<T>")
        .register_type_size::<Balance>("BalanceOf<T>")
        .skip_type_sizes_check()
        .build()
        .await
        .unwrap();

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
    rely_subxt_client: &Client<RelayRuntime>,
    para_signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    multi_account_id: AccountId,
    threshold: u16,
    others: Vec<AccountId>,
    first: bool,
) {
    loop {
        match system_rpc_rx.recv().await {
            Some((task_type, response)) => {
                match task_type {
                    TasksType::ParaStake(amount) => {
                        let _ = start_withdraw_task_para(
                            &para_subxt_client,
                            &rely_subxt_client,
                            para_signer,
                            multi_account_id.clone(),
                            threshold.clone(),
                            others.clone(),
                            first.clone(),
                            amount.clone(),
                        )
                        .await
                        .map_err(|e| println!("error start_withdraw_task_para: {:?}", e));
                        response.send(0).unwrap();
                    }
                    TasksType::ParaUnstake(_amount) => {
                        // relay_bond_extra(
                        //     subxt_relay_client,
                        //     relay_signer,
                        //     others.clone(),
                        //     pool_addr.clone(),
                        //     first,
                        // ).await
                        //     .map_err(|e| println!("error do_last_para_record_rewards: {:?}", e));
                        response.send(0).unwrap();
                    }
                }
            }
            None => println!("dispatch pending..."),
        }
    }
}

/// start withdraw task, ws_server: ws://127.0.0.1:9944
pub(crate) async fn start_withdraw_task_para(
    para_subxt_client: &Client<HeikoRuntime>,
    relay_subxt_client: &Client<RelayRuntime>,
    para_signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    multi_account_id: AccountId,
    threshold: u16,
    others: Vec<AccountId>,
    first: bool,
    amount: Amount,
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

        // todo transfer relay chain amount from one address to other
        let _ = transfer_relay_chain_balance(&relay_subxt_client, amount.clone()).await?;
        println!("[+] Create mock relay transaction finished");
    }
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
