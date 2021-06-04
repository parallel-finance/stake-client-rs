use crate::listener::listen_pool_balances;
use crate::primitives::{AccountId, MIN_POOL_BALANCE};

use crate::DB;
use async_std::task;
use core::marker::PhantomData;
use db::executor::DbExecutor;
use db::model::WithdrawTx;
use db::schema::withdraw_tx::dsl::*;
use diesel::{
    self, query_dsl::BelongingToDsl, BoolExpressionMethods, Connection, ExpressionMethods,
    QueryDsl, RunQueryDsl,
};
use parallel_primitives::CurrencyId;
use runtime::error::Error;
use runtime::heiko::{self, runtime::HeikoRuntime};
use runtime::kusama::{self, runtime::KusamaRuntime as RelayRuntime};
use runtime::pallets::multisig::{ApproveAsMultiCall, AsMultiCall, Multisig, Timepoint};
use sp_core::sr25519::Pair;
use sp_core::{crypto::Pair as TraitPair, crypto::Ss58Codec};
use sp_keyring::AccountKeyring;
use std::str::FromStr;
use std::time::Duration;
use std::{thread, time};
use substrate_subxt::Error as SubError;
use substrate_subxt::{
    balances, sp_runtime::traits::IdentifyAccount, staking, sudo, Client, ClientBuilder, Encoded,
    PairSigner, Runtime, Signer,
};

const DEFAULT_WS_SERVER: &str = "ws://127.0.0.1:9944";
const DEFAULT_POOL_ADDR: &str = "5DjYJStmdZ2rcqXbXGX7TW85JsrW6uG4y9MUcLq2BoPMpRA7";

/// start withdraw task, ws_server: ws://127.0.0.1:9944
pub(crate) async fn start_withdraw_task(
    pair: Pair,
    others: Vec<AccountId>,
    mut ws_server: &str,
    mut pool_addr: &str,
    first: bool, // temp use
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

    loop {
        // todo complete me
        println!("start listen_pool_balances");
        let _ = listen_pool_balances(
            subxt_client.clone(),
            ws_server.clone(),
            pool_addr.clone(),
            CurrencyId::KSM,
        )
        .await;

        // todo get state from db
        // let conn = DB.get_connection().unwrap();
        // let rr = withdraw_tx.load::<WithdrawTx>(&conn).unwrap();

        // conn.

        let signer = PairSigner::<HeikoRuntime, Pair>::new(pair.clone());
        if first {
            let (account_id, call_hash) = do_first_withdraw(
                others.clone(),
                ws_server.clone(),
                pool_addr.clone(),
                &subxt_client,
                &signer,
            )
            .await?;
            let _ = wait_transfer_finished(&subxt_client, account_id, call_hash).await?;
            // todo wait transfer finished and update db
        } else {
            let (account_id, call_hash) = do_last_withdraw(
                others.clone(),
                ws_server.clone(),
                pool_addr.clone(),
                &subxt_client,
                &signer,
            )
            .await?;
            let _ = wait_transfer_finished(&subxt_client, account_id, call_hash).await?;
            // todo wait transfer finished and update db
        }
        //task::block_on(do_middle_withdraw());
    }

    Ok(())
}

/// The first wallet to call withdraw. No need use 'TimePoint' and call 'approve_as_multi'.
pub(crate) async fn do_first_withdraw(
    others: Vec<AccountId>,
    ws_server: &str,
    pool_addr: &str,
    subxt_client: &Client<HeikoRuntime>,
    signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
) -> Result<(AccountId, [u8; 32]), Error> {
    println!("do_first_withdraw");

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
    Ok((account_id, call_hash))
}

/// If the wallet is the middle one to call withdraw, need to get 'TimePoint' and call 'approve_as_multi'.
pub(crate) async fn do_middle_withdraw() -> Result<(), Error> {
    println!("do_middle_withdraw");
    Ok(())
}

/// If the wallet is the last one need to get 'TimePoint' and call 'as_multi'.
pub(crate) async fn do_last_withdraw(
    others: Vec<AccountId>,
    ws_server: &str,
    pool_addr: &str,
    subxt_client: &Client<HeikoRuntime>,
    signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
) -> Result<(AccountId, [u8; 32]), Error> {
    println!("do_last_withdraw");
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
    Ok((account_id, call_hash))
}

pub(crate) async fn get_last_withdraw_time_point(
    subxt_client: &Client<HeikoRuntime>,
    account_id: AccountId,
    call_hash: [u8; 32],
) -> Result<Timepoint<u32>, Error> {
    loop {
        println!("get time point, waiting...");
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
