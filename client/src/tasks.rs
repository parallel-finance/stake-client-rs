use crate::listener::listen_pool_balances;
use crate::primitives::{AccountId, MIN_POOL_BALANCE};

use crate::DB;
use chrono::{NaiveDateTime, Utc};
use db::executor::DbExecutor;
use db::model::Withdraw;
use db::schema::withdraw::dsl::*;
use diesel::{
    self, insert_into, query_dsl::BelongingToDsl, update, BoolExpressionMethods, Connection,
    ExpressionMethods, QueryDsl, RunQueryDsl,
};

use runtime::error::Error;
use runtime::heiko::{self, runtime::HeikoRuntime};
use runtime::kusama::{self};
use runtime::pallets::multisig::Timepoint;
use sp_core::crypto::Ss58Codec;
use sp_core::sr25519::Pair;
use sp_keyring::AccountKeyring;
use std::{thread, time};
use substrate_subxt::Error as SubError;
use substrate_subxt::{balances, Client, ClientBuilder, PairSigner, Signer};

// const DEFAULT_WS_SERVER: &str = "ws://127.0.0.1:9944";
// const DEFAULT_POOL_ADDR: &str = "5DjYJStmdZ2rcqXbXGX7TW85JsrW6uG4y9MUcLq2BoPMpRA7";

/// start withdraw task, ws_server: ws://127.0.0.1:9944
pub(crate) async fn start_withdraw_task(
    pair: Pair,
    others: Vec<AccountId>,
    ws_server: &str,
    pool_addr: &str,
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

    let mut current_withdraw_index: i32 = 0;
    let conn = DB.get_connection().map_err(|e| {
        println!("failed to connect DB, error: {:?}", e);
        Error::SubxtError(SubError::Other("failed to connect DB".to_string()))
    })?;

    loop {
        let r = withdraw.load::<Withdraw>(&conn).map_err(|e| {
            println!("\r\rfailed to load Withdraw table, error: {:?}", e);
            Error::SubxtError(SubError::Other("failed to load Withdraw table".to_string()))
        })?;
        current_withdraw_index = r.len() as i32;
        println!("current index:{}", current_withdraw_index);
        if current_withdraw_index != 0 {
            println!("last withdraw record:{:?}", r[r.len() - 1]);
        }

        println!("[+] Listen to pool's balance");
        let _ =
            listen_pool_balances(subxt_client.clone(), ws_server.clone(), pool_addr.clone()).await;

        let withdraw_creating = Withdraw {
            idx: current_withdraw_index.clone(),
            state: "creating".to_string(),
            created_at: Utc::now().naive_utc(),
        };
        let first: bool;
        match insert_into(withdraw)
            .values(&withdraw_creating)
            .execute(&conn)
        {
            Ok(_) => first = true,
            Err(_) => first = false,
        }

        let signer = PairSigner::<HeikoRuntime, Pair>::new(pair.clone());
        if first {
            let (account_id, call_hash) =
                do_first_withdraw(others.clone(), pool_addr.clone(), &subxt_client, &signer)
                    .await?;
            let _ = wait_transfer_finished(&subxt_client, account_id, call_hash).await?;
            println!("[+] Create withdraw transaction finished")
        } else {
            let (account_id, call_hash) =
                do_last_withdraw(others.clone(), pool_addr.clone(), &subxt_client, &signer).await?;
            let _ = wait_transfer_finished(&subxt_client, account_id, call_hash).await?;

            // the second wallet nee to update withdraw db
            // todo use all client to deal with withdraw work
            let withdraw_created = Withdraw {
                idx: current_withdraw_index.clone(),
                state: "created".to_string(),
                created_at: Utc::now().naive_utc(),
            };
            match update(withdraw.filter(idx.eq(current_withdraw_index.clone())))
                .set(&withdraw_created)
                .execute(&conn)
            {
                Ok(con) => println!("update withdraw db finished"),
                Err(_) => println!("failed to update withdraw"),
            }
            println!("[+] Create withdraw transaction finished")
        }
        //task::block_on(do_middle_withdraw());
    }
}

/// The first wallet to call withdraw. No need use 'TimePoint' and call 'approve_as_multi'.
pub(crate) async fn do_first_withdraw(
    others: Vec<AccountId>,
    pool_addr: &str,
    subxt_client: &Client<HeikoRuntime>,
    signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
) -> Result<(AccountId, [u8; 32]), Error> {
    println!("[+] Create first withdraw transaction");

    println!("---------- start create multi-signature transaction ----------");
    // 1.1 construct balance transfer call
    // todo change balances_transfer to withdraw
    let dest = AccountKeyring::Eve.to_account_id().into();
    let call = heiko::api::balances_transfer_call::<HeikoRuntime>(&dest, MIN_POOL_BALANCE);
    let mc = heiko::api::multisig_approve_as_multi_call::<
        HeikoRuntime,
        balances::TransferCall<HeikoRuntime>,
    >(subxt_client, 2, others, None, call.clone(), 0u64)?;
    // 1.2 initial the multisg call
    let result = subxt_client.submit(mc, signer).await?;
    println!("multisig_approve_as_multi_call hash {:?}", result);

    // get account_id of pool
    let account_id = AccountId::from_string(pool_addr)
        .map_err(|_| SubError::Other("invalid pool address".to_string()))?;
    let call_hash = kusama::api::multisig_call_hash(subxt_client, call.clone())?;
    println!("---------- end create multi-signature transaction ----------");
    Ok((account_id, call_hash))
}

/// If the wallet is the middle one to call withdraw, need to get 'TimePoint' and call 'approve_as_multi'.
// pub(crate) async fn do_middle_withdraw() -> Result<(), Error> {
//     println!("do_middle_withdraw");
//     Ok(())
// }

/// If the wallet is the last one need to get 'TimePoint' and call 'as_multi'.
pub(crate) async fn do_last_withdraw(
    others: Vec<AccountId>,
    pool_addr: &str,
    subxt_client: &Client<HeikoRuntime>,
    signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
) -> Result<(AccountId, [u8; 32]), Error> {
    println!("[+] Crate last withdraw transaction");
    println!("---------- start create multi-signature transaction ----------");
    // 1.1 construct balance transfer call
    // todo change blances_transfer to withdraw
    let account_id = AccountId::from_string(pool_addr)
        .map_err(|_| SubError::Other("invalid pool address".to_string()))?;
    let dest = AccountKeyring::Eve.to_account_id().into();
    let call = heiko::api::balances_transfer_call::<HeikoRuntime>(&dest, MIN_POOL_BALANCE);
    let call_hash = kusama::api::multisig_call_hash(subxt_client, call.clone())?;
    let when = get_last_withdraw_time_point(subxt_client, account_id.clone(), call_hash).await?;
    println!("multisig timepoint{:?}", when);

    let mc =
        kusama::api::multisig_as_multi_call::<HeikoRuntime, balances::TransferCall<HeikoRuntime>>(
            subxt_client,
            2,
            others,
            Some(Timepoint::new(when.height, when.index)),
            call,
            false,
            1_000_000_000_000,
        )?;
    // 1.2 initial the multisg call
    let result = subxt_client.submit(mc, signer).await?;
    println!("multisig_as_multi_call hash {:?}", result);
    println!("---------- end create multi-signature transaction ----------");
    Ok((account_id, call_hash))
}

pub(crate) async fn get_last_withdraw_time_point(
    subxt_client: &Client<HeikoRuntime>,
    account_id: AccountId,
    call_hash: [u8; 32],
) -> Result<Timepoint<u32>, Error> {
    println!("get time point, waiting...");
    loop {
        let store = kusama::api::MultisigsStore::<HeikoRuntime> {
            multisig_account: account_id.clone(),
            call_hash: call_hash.clone(),
        };
        if let Some(kusama::api::MultisigData { when, .. }) =
            subxt_client.fetch(&store, None).await?
        {
            return Ok(when);
        }
        thread::sleep(time::Duration::from_secs(1));
    }
}

pub(crate) async fn wait_transfer_finished(
    subxt_client: &Client<HeikoRuntime>,
    account_id: AccountId,
    call_hash: [u8; 32],
) -> Result<(), Error> {
    // todo check if the transaction is in block
    println!("transferring, waiting...");
    thread::sleep(time::Duration::from_secs(20));

    loop {
        let store = kusama::api::MultisigsStore::<HeikoRuntime> {
            multisig_account: account_id.clone(),
            call_hash: call_hash.clone(),
        };
        if let Some(kusama::api::MultisigData { when: _, .. }) =
            subxt_client.fetch(&store, None).await?
        {
            let times = time::Duration::from_secs(5);
            thread::sleep(times);
        } else {
            break Ok(());
        }
    }
}
