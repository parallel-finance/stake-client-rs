use crate::listener::listen_pool_balances;
use crate::primitives::{AccountId, MIN_WITHDRAW_BALANCE};
use crate::tasks::{
    do_first_withdraw, do_last_withdraw, get_pool_balances, get_withdraw_call_hash,
    wait_transfer_finished,
};

use crate::DB;
use chrono::Utc;
use db::model::Withdraw;
use db::schema::withdraw::dsl::*;
use diesel::{self, insert_into, update, ExpressionMethods, QueryDsl, RunQueryDsl};

use core::marker::PhantomData;
use parallel_primitives::CurrencyId;
use runtime::error::Error;
use runtime::heiko::{self, runtime::HeikoRuntime};
use runtime::kusama::{self};
use runtime::pallets::multisig::Multisig;
use sp_core::crypto::Ss58Codec;
use sp_core::sr25519::Pair;
use sp_keyring::AccountKeyring;
use std::{thread, time};
use substrate_subxt::{sudo, Client, ClientBuilder, PairSigner, Signer};
use substrate_subxt::{Error as SubError, Runtime};

/// start withdraw task, ws_server: ws://127.0.0.1:9944
pub(crate) async fn start_withdraw_task_para(
    threshold: u16,
    pair: Pair,
    others: Vec<AccountId>,
    ws_server: &str,
    pool_addr: &str,
    multi_addr: &str,
    currency_id: CurrencyId,
    first: bool,
) -> Result<(), Error> {
    // initialize heiko related api
    let subxt_client = ClientBuilder::<HeikoRuntime>::new()
        .set_url(ws_server)
        .skip_type_sizes_check()
        .build()
        .await
        .map_err(|e| {
            println!("subxt_client error: {:?}", e);
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
            subxt_client.clone(),
            pool_account_id.clone(),
            currency_id.clone(),
        )
        .await
        {
            Ok(a) => amount = a,
            Err(_) => continue,
        }

        let signer = PairSigner::<HeikoRuntime, Pair>::new(pair.clone());
        if first {
            let call_hash =
                do_first_withdraw(others.clone(), &subxt_client, &signer, amount).await?;
            let _ =
                wait_transfer_finished(&subxt_client, multi_account_id.clone(), call_hash).await?;
            println!("[+] Create withdraw transaction finished")
        } else {
            let call_hash = do_last_withdraw(
                others.clone(),
                multi_account_id.clone(),
                &subxt_client,
                &signer,
                amount,
            )
            .await?;
            let _ =
                wait_transfer_finished(&subxt_client, multi_account_id.clone(), call_hash).await?;
            println!("[+] Create withdraw transaction finished")
        }
    }
}
