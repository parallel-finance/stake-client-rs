use crate::listener::listen_pool_balances;
use crate::primitives::AccountId;
use crate::tasks::{do_first_withdraw, do_last_withdraw, wait_transfer_finished};

use parallel_primitives::CurrencyId;
use runtime::error::Error;
use runtime::heiko::runtime::HeikoRuntime;
use runtime::kusama::{self, runtime::KusamaRuntime as RelayRuntime};
use sp_core::crypto::Ss58Codec;
use sp_core::Pair;
use substrate_subxt::Error as SubError;
use substrate_subxt::{Client, ClientBuilder, PairSigner};

const FROM_RELAY_CHAIN_SEED: &str = "//Alice";
const TO_REPLAY_CHAIN_ADDRESS: &str = "5DjYJStmdZ2rcqXbXGX7TW85JsrW6uG4y9MUcLq2BoPMpRA7";

/// start withdraw task, ws_server: ws://127.0.0.1:9944
pub(crate) async fn start_withdraw_task_para(
    _threshold: u16,
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
        .skip_type_sizes_check()
        .build()
        .await
        .map_err(|e| {
            println!("para_subxt_client error: {:?}", e);
            Error::SubxtError(e)
        })?;

    let relay_subxt_client = ClientBuilder::<RelayRuntime>::new()
        .set_url(relay_ws_server)
        .skip_type_sizes_check()
        // .register_type_size::<([u8; 20])>("EthereumAddress")
        .build()
        .await
        .map_err(|e| {
            println!("relay_subxt_client error: {:?}", e);
            Error::SubxtError(e)
        })?;

    let multi_account_id = AccountId::from_string(multi_addr)
        .map_err(|_| SubError::Other("invalid pool address".to_string()))?;

    let pool_account_id = AccountId::from_string(pool_addr)
        .map_err(|_| SubError::Other("invalid pool address".to_string()))?;

    loop {
        println!("[+] Listen to pool's balance");
        // todo check state, finished?
        let amount: u128;
        match listen_pool_balances(
            para_subxt_client.clone(),
            pool_account_id.clone(),
            currency_id.clone(),
        )
        .await
        {
            Ok(a) => amount = a,
            Err(_) => continue,
        }

        let signer = PairSigner::<HeikoRuntime, sp_core::sr25519::Pair>::new(pair.clone());
        if first {
            let call_hash =
                do_first_withdraw(others.clone(), &para_subxt_client, &signer, amount).await?;
            let _ = wait_transfer_finished(&para_subxt_client, multi_account_id.clone(), call_hash)
                .await?;
            println!("[+] Create withdraw transaction finished")
        } else {
            let call_hash = do_last_withdraw(
                others.clone(),
                multi_account_id.clone(),
                &para_subxt_client,
                &signer,
                amount,
            )
            .await?;
            let _ = wait_transfer_finished(&para_subxt_client, multi_account_id.clone(), call_hash)
                .await?;
            println!("[+] Create withdraw transaction finished");

            // todo transfer relay chain amount from one address to other
            let _ = transfer_relay_chain_balance(&relay_subxt_client, amount.clone()).await?;
        }
    }
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
