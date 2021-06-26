// use crate::listener::listen_pool_balances;
use crate::primitives::{AccountId, Amount, MIN_WITHDRAW_BALANCE};

// use crate::DB;
// use chrono::Utc;
// use db::model::Withdraw;
// use db::schema::withdraw::dsl::*;
// use diesel::{self, insert_into, update, ExpressionMethods, QueryDsl, RunQueryDsl};

use core::marker::PhantomData;
use parallel_primitives::CurrencyId;
use runtime::error::Error;
use runtime::heiko::{self, runtime::HeikoRuntime};
use runtime::kusama::{self};
use runtime::pallets::multisig::Multisig;
// use sp_core::crypto::Ss58Codec;
use sp_core::sr25519::Pair;
use sp_keyring::AccountKeyring;
use std::{thread, time};
use substrate_subxt::{sudo, Client, Signer};
use substrate_subxt::{Error as SubError, Runtime};

// const DEFAULT_WS_SERVER: &str = "ws://127.0.0.1:9944";
// const DEFAULT_POOL_ADDR: &str = "5DjYJStmdZ2rcqXbXGX7TW85JsrW6uG4y9MUcLq2BoPMpRA7";

/// start withdraw task, ws_server: ws://127.0.0.1:9944
pub(crate) async fn start_withdraw_task(
    _threshold: u16,
    _pair: Pair,
    _others: Vec<AccountId>,
    _ws_server: &str,
    _pool_addr: &str,
    _multi_addr: &str,
    _currency_id: CurrencyId,
) -> Result<(), Error> {
    // // initialize heiko related api
    // let subxt_client = ClientBuilder::<HeikoRuntime>::new()
    //     .set_url(ws_server)
    //     .skip_type_sizes_check()
    //     .build()
    //     .await
    //     .map_err(|e| {
    //         println!("subxt_client error: {:?}", e);
    //         Error::SubxtError(e)
    //     })?;
    //
    // let mut current_withdraw_index: i32;
    // let conn = DB.get_connection().map_err(|e| {
    //     println!("failed to connect DB, error: {:?}", e);
    //     Error::SubxtError(SubError::Other("failed to connect DB".to_string()))
    // })?;
    //
    // let multi_account_id = AccountId::from_string(multi_addr)
    //     .map_err(|_| SubError::Other("invalid pool address".to_string()))?;
    //
    // let pool_account_id = AccountId::from_string(pool_addr)
    //     .map_err(|_| SubError::Other("invalid pool address".to_string()))?;
    //
    // loop {
    //     let table_withdraw = withdraw.load::<Withdraw>(&conn).map_err(|e| {
    //         println!("failed to load Withdraw table, error: {:?}", e);
    //         Error::SubxtError(SubError::Other("failed to load Withdraw table".to_string()))
    //     })?;
    //
    //     current_withdraw_index = table_withdraw.len() as i32;
    //     let mut creating: bool = false;
    //     if current_withdraw_index != 0 {
    //         println!(
    //             "last withdraw record:{:?}",
    //             table_withdraw[table_withdraw.len() - 1]
    //         );
    //         if table_withdraw[table_withdraw.len() - 1].state == "creating".to_string() {
    //             creating = true
    //         }
    //     }
    //     println!("current index:{}", current_withdraw_index);
    //
    //     let mut balance_enough: bool = false;
    //     if let Some(balance) = get_pool_balances(
    //         subxt_client.clone(),
    //         pool_account_id.clone(),
    //         currency_id.clone(),
    //     )
    //     .await
    //     {
    //         if balance > MIN_WITHDRAW_BALANCE {
    //             balance_enough = true
    //         }
    //     }
    //
    //     if balance_enough && creating {
    //         current_withdraw_index = current_withdraw_index - 1;
    //     }
    //
    //     println!("[+] Listen to pool's balance");
    //     // todo check state, finished?
    //     let amount: u128;
    //     match listen_pool_balances(
    //         subxt_client.clone(),
    //         pool_account_id.clone(),
    //         currency_id.clone(),
    //     )
    //     .await
    //     {
    //         Ok(a) => amount = a,
    //         Err(_) => continue,
    //     }
    //
    //     let withdraw_creating = Withdraw {
    //         idx: current_withdraw_index.clone(),
    //         state: "creating".to_string(),
    //         created_at: Utc::now().naive_utc(),
    //         sig_count: Some(1),
    //         height: Some(0),
    //     };
    //     let first: bool;
    //     match insert_into(withdraw)
    //         .values(&withdraw_creating)
    //         .execute(&conn)
    //     {
    //         Ok(_) => first = true,
    //         Err(_) => first = false,
    //     }
    //
    //     let signer = PairSigner::<HeikoRuntime, Pair>::new(pair.clone());
    //     if first {
    //         let call_hash =
    //             do_first_withdraw(others.clone(), &subxt_client, &signer, amount).await?;
    //         let _ =
    //             wait_transfer_finished(&subxt_client, multi_account_id.clone(), call_hash).await?;
    //         println!("[+] Create withdraw transaction finished")
    //     } else {
    //         let mut finished: bool = false;
    //         let mut last: bool = false;
    //         let mut signature_count: i32 = 0;
    //
    //         // try to get current sig_count and update to sig_count+1
    //         loop {
    //             let sc: Option<i32> = withdraw
    //                 .filter(idx.eq(current_withdraw_index.clone()))
    //                 .select(sig_count)
    //                 .first(&conn)
    //                 .map_err(|_| SubError::Other("failed to get sig count".to_string()))?;
    //             let count: i32;
    //             match sc {
    //                 Some(c) => count = c,
    //                 None => count = 0,
    //             }
    //
    //             let withdraw_created = Withdraw {
    //                 idx: current_withdraw_index.clone(),
    //                 state: "created".to_string(),
    //                 sig_count: Some(count.clone() + 1),
    //                 height: Some(0),
    //                 created_at: Utc::now().naive_utc(),
    //             };
    //
    //             if count + 1 > threshold as i32 {
    //                 finished = true;
    //                 break;
    //             }
    //
    //             match update(withdraw.filter(idx.eq(current_withdraw_index.clone())))
    //                 .filter(sig_count.eq(count.clone()))
    //                 .set(&withdraw_created)
    //                 .execute(&conn)
    //             {
    //                 Ok(_con) => {
    //                     last = count + 1 == threshold as i32;
    //                     signature_count = count.clone() + 1;
    //                     println!(
    //                         "update withdraw db finished, last:{:?}, current signature:{:?}",
    //                         last, signature_count
    //                     );
    //                     break;
    //                 }
    //                 Err(_) => println!(
    //                     "current sig_count:{:?}, failed to update withdraw, try again...",
    //                     count
    //                 ),
    //             }
    //             thread::sleep(time::Duration::from_secs(1));
    //         }
    //
    //         if finished {
    //             let call_hash = get_withdraw_call_hash(&subxt_client).await?;
    //             let _ = wait_transfer_finished(&subxt_client, multi_account_id.clone(), call_hash)
    //                 .await?;
    //             println!("signature is enough, no need to create transaction.");
    //         } else if last {
    //             let call_hash = do_last_withdraw(
    //                 others.clone(),
    //                 multi_account_id.clone(),
    //                 &subxt_client,
    //                 &signer,
    //                 amount,
    //             )
    //             .await?;
    //             let _ = wait_transfer_finished(&subxt_client, multi_account_id.clone(), call_hash)
    //                 .await?;
    //
    //             // the second wallet nee to update withdraw db
    //             // todo use all client to deal with withdraw work
    //             let withdraw_created = Withdraw {
    //                 idx: current_withdraw_index.clone(),
    //                 state: "created".to_string(),
    //                 sig_count: Some(signature_count),
    //                 height: Some(0),
    //                 created_at: Utc::now().naive_utc(),
    //             };
    //             match update(withdraw.filter(idx.eq(current_withdraw_index.clone())))
    //                 .set(&withdraw_created)
    //                 .execute(&conn)
    //             {
    //                 Ok(_con) => println!("update withdraw db finished"),
    //                 Err(_) => println!("failed to update withdraw"),
    //             }
    //             println!("[+] Create withdraw transaction finished")
    //         } else {
    //             // todo do middle_withdraw
    //             let call_hash = do_last_withdraw(
    //                 others.clone(),
    //                 multi_account_id.clone(),
    //                 &subxt_client,
    //                 &signer,
    //                 amount,
    //             )
    //             .await?;
    //             let _ = wait_transfer_finished(&subxt_client, multi_account_id.clone(), call_hash)
    //                 .await?;
    //             println!(
    //                 "[+] Create withdraw transaction count:{:?}",
    //                 signature_count
    //             )
    //         }
    //     }
    // }
    Ok(())
}

/// The first wallet to call withdraw. No need use 'TimePoint' and call 'approve_as_multi'.
pub(crate) async fn do_first_withdraw(
    others: Vec<AccountId>,
    subxt_client: &Client<HeikoRuntime>,
    signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    amount: Amount,
    threshold: u16,
) -> Result<[u8; 32], Error> {
    println!("[+] Create first withdraw transaction");

    println!("---------- start create multi-signature transaction ----------");
    // 1.1 construct balance transfer call
    let dest = AccountKeyring::Eve.to_account_id().into();
    let inner_call = heiko::api::liquid_staking_withdraw_call::<HeikoRuntime>(dest, amount);
    let inner_call_encoded = subxt_client
        .encode(inner_call)
        .map_err(|e| Error::SubxtError(e))?;
    let sudo_call = sudo::SudoCall::<HeikoRuntime> {
        _runtime: PhantomData,
        call: &inner_call_encoded,
    };

    let mc =
        heiko::api::multisig_approve_as_multi_call::<HeikoRuntime, sudo::SudoCall<HeikoRuntime>>(
            subxt_client,
            threshold,
            others,
            None,
            sudo_call.clone(),
            0u64,
        )?;

    // 1.2 initial the multisg call
    let result = subxt_client.submit(mc, signer).await?;
    println!("multisig_approve_as_multi_call hash {:?}", result);

    // get account_id of multi address
    let call_hash = kusama::api::multisig_call_hash(subxt_client, sudo_call)?;
    println!("call hash {:?}", format!("0x{}", hex::encode(call_hash)));
    println!("---------- end create multi-signature transaction ----------");
    Ok(call_hash)
}

/// If the wallet is the middle one to call withdraw, need to get 'TimePoint' and call 'approve_as_multi'.
// pub(crate) async fn do_middle_withdraw() -> Result<(), Error> {
//     println!("do_middle_withdraw");
//     Ok(())
// }

/// If the wallet is the last one need to get 'TimePoint' and call 'as_multi'.
pub(crate) async fn do_last_withdraw(
    others: Vec<AccountId>,
    multi_account_id: AccountId,
    subxt_client: &Client<HeikoRuntime>,
    signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    amount: Amount,
    threshold: u16,
) -> Result<[u8; 32], Error> {
    println!("[+] Crate last withdraw transaction");
    println!("---------- start create multi-signature transaction ----------");
    // 1.1 construct balance transfer call
    let dest = AccountKeyring::Eve.to_account_id().into();
    let inner_call = heiko::api::liquid_staking_withdraw_call::<HeikoRuntime>(dest, amount);

    let inner_call_encoded = subxt_client
        .encode(inner_call)
        .map_err(|e| Error::SubxtError(e))?;
    let sudo_call = sudo::SudoCall::<HeikoRuntime> {
        _runtime: PhantomData,
        call: &inner_call_encoded,
    };

    // check if timepoint already exist.
    let call_hash =
        heiko::api::multisig_call_hash(subxt_client, sudo_call.clone()).map_err(|_e| {
            Error::SubxtError(SubError::Other("failed to load get call hash".to_string()))
        })?;

    // let when = get_time_point::<HeikoRuntime>(subxt_client, account_id.clone(), call_hash).await;
    let when =
        get_last_time_point::<HeikoRuntime>(subxt_client, multi_account_id.clone(), call_hash)
            .await;
    println!("multisig timepoint{:?}", when);

    let mc = kusama::api::multisig_as_multi_call::<HeikoRuntime, sudo::SudoCall<HeikoRuntime>>(
        subxt_client,
        threshold,
        others,
        when,
        sudo_call,
        false,
        1_000_000_000_000,
    )?;

    // 1.2 initial the multisg call
    let result = subxt_client.submit(mc, signer).await?;
    println!("multisig_as_multi_call hash {:?}", result);
    println!("---------- end create multi-signature transaction ----------");
    Ok(call_hash)
}

/// The first wallet to call process_pending_unstake. No need use 'TimePoint' and call 'approve_as_multi'.
pub(crate) async fn do_first_process_pending_unstake(
    others: Vec<AccountId>,
    subxt_client: &Client<HeikoRuntime>,
    signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    agent: AccountId,
    owner: AccountId,
    amount: Amount,
    threshold: u16,
) -> Result<[u8; 32], Error> {
    println!("[+] Create first process_pending_unstake transaction");

    println!("---------- start create multi-signature transaction ----------");
    // 1.1 construct balance transfer call
    let inner_call = heiko::api::liquid_staking_process_pending_unstake_call::<HeikoRuntime>(
        agent, owner, amount,
    );
    let inner_call_encoded = subxt_client
        .encode(inner_call)
        .map_err(|e| Error::SubxtError(e))?;
    let sudo_call = sudo::SudoCall::<HeikoRuntime> {
        _runtime: PhantomData,
        call: &inner_call_encoded,
    };

    let mc =
        heiko::api::multisig_approve_as_multi_call::<HeikoRuntime, sudo::SudoCall<HeikoRuntime>>(
            subxt_client,
            threshold,
            others,
            None,
            sudo_call.clone(),
            0u64,
        )?;

    // 1.2 initial the multisg call
    let result = subxt_client.submit(mc, signer).await?;
    println!("multisig_approve_as_multi_call hash {:?}", result);

    // get account_id of multi address
    let call_hash = kusama::api::multisig_call_hash(subxt_client, sudo_call)?;
    println!("call hash {:?}", format!("0x{}", hex::encode(call_hash)));
    println!("---------- end create multi-signature transaction ----------");
    Ok(call_hash)
}

/// If the wallet is the last one need to get 'TimePoint' and call 'as_multi'.
pub(crate) async fn do_last_process_pending_unstake(
    others: Vec<AccountId>,
    multi_account_id: AccountId,
    subxt_client: &Client<HeikoRuntime>,
    signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    agent: AccountId,
    owner: AccountId,
    amount: Amount,
    threshold: u16,
) -> Result<[u8; 32], Error> {
    println!("[+] Crate last process_pending_unstake transaction");
    println!("---------- start create multi-signature transaction ----------");
    // construct process pending unstake call
    let inner_call = heiko::api::liquid_staking_process_pending_unstake_call::<HeikoRuntime>(
        agent, owner, amount,
    );

    let inner_call_encoded = subxt_client
        .encode(inner_call)
        .map_err(|e| Error::SubxtError(e))?;
    let sudo_call = sudo::SudoCall::<HeikoRuntime> {
        _runtime: PhantomData,
        call: &inner_call_encoded,
    };

    // check if timepoint already exist.
    let call_hash =
        heiko::api::multisig_call_hash(subxt_client, sudo_call.clone()).map_err(|_e| {
            Error::SubxtError(SubError::Other("failed to load get call hash".to_string()))
        })?;

    // let when = get_time_point::<HeikoRuntime>(subxt_client, account_id.clone(), call_hash).await;
    let when =
        get_last_time_point::<HeikoRuntime>(subxt_client, multi_account_id.clone(), call_hash)
            .await;
    println!("multisig timepoint{:?}", when);

    let mc = kusama::api::multisig_as_multi_call::<HeikoRuntime, sudo::SudoCall<HeikoRuntime>>(
        subxt_client,
        threshold,
        others,
        when,
        sudo_call,
        false,
        1_000_000_000_000,
    )?;

    // 1.2 initial the multisg call
    let result = subxt_client.submit(mc, signer).await?;
    println!("multisig_as_multi_call hash {:?}", result);
    println!("---------- end create multi-signature transaction ----------");
    Ok(call_hash)
}

/// The first wallet to call finish_processed_unstake. No need use 'TimePoint' and call 'approve_as_multi'.
pub(crate) async fn do_first_finish_processed_unstake(
    others: Vec<AccountId>,
    subxt_client: &Client<HeikoRuntime>,
    signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    agent: AccountId,
    owner: AccountId,
    amount: Amount,
    threshold: u16,
) -> Result<[u8; 32], Error> {
    println!("[+] Create first finish_processed_unstake transaction");

    println!("---------- start create multi-signature transaction ----------");
    // 1.1 construct balance transfer call
    let inner_call = heiko::api::liquid_staking_finish_processed_unstake_call::<HeikoRuntime>(
        agent, owner, amount,
    );
    let inner_call_encoded = subxt_client
        .encode(inner_call)
        .map_err(|e| Error::SubxtError(e))?;
    let sudo_call = sudo::SudoCall::<HeikoRuntime> {
        _runtime: PhantomData,
        call: &inner_call_encoded,
    };

    let mc =
        heiko::api::multisig_approve_as_multi_call::<HeikoRuntime, sudo::SudoCall<HeikoRuntime>>(
            subxt_client,
            threshold,
            others,
            None,
            sudo_call.clone(),
            0u64,
        )?;

    // 1.2 initial the multisg call
    let result = subxt_client.submit(mc, signer).await?;
    println!("multisig_approve_as_multi_call hash {:?}", result);

    // get account_id of multi address
    let call_hash = kusama::api::multisig_call_hash(subxt_client, sudo_call)?;
    println!("call hash {:?}", format!("0x{}", hex::encode(call_hash)));
    println!("---------- end create multi-signature transaction ----------");
    Ok(call_hash)
}

/// If the wallet is the last one need to get 'TimePoint' and call 'as_multi'.
pub(crate) async fn do_last_finish_processed_unstake(
    others: Vec<AccountId>,
    multi_account_id: AccountId,
    subxt_client: &Client<HeikoRuntime>,
    signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    agent: AccountId,
    owner: AccountId,
    amount: Amount,
    threshold: u16,
) -> Result<[u8; 32], Error> {
    println!("[+] Crate last finish_processed_unstake transaction");
    println!("---------- start create multi-signature transaction ----------");
    // construct process pending unstake call
    let inner_call = heiko::api::liquid_staking_finish_processed_unstake_call::<HeikoRuntime>(
        agent, owner, amount,
    );

    let inner_call_encoded = subxt_client
        .encode(inner_call)
        .map_err(|e| Error::SubxtError(e))?;
    let sudo_call = sudo::SudoCall::<HeikoRuntime> {
        _runtime: PhantomData,
        call: &inner_call_encoded,
    };

    // check if timepoint already exist.
    let call_hash =
        heiko::api::multisig_call_hash(subxt_client, sudo_call.clone()).map_err(|_e| {
            Error::SubxtError(SubError::Other("failed to load get call hash".to_string()))
        })?;

    // let when = get_time_point::<HeikoRuntime>(subxt_client, account_id.clone(), call_hash).await;
    let when =
        get_last_time_point::<HeikoRuntime>(subxt_client, multi_account_id.clone(), call_hash)
            .await;
    println!("multisig timepoint{:?}", when);

    let mc = kusama::api::multisig_as_multi_call::<HeikoRuntime, sudo::SudoCall<HeikoRuntime>>(
        subxt_client,
        threshold,
        others,
        when,
        sudo_call,
        false,
        1_000_000_000_000,
    )?;

    // 1.2 initial the multisg call
    let result = subxt_client.submit(mc, signer).await?;
    println!("multisig_as_multi_call hash {:?}", result);
    println!("---------- end create multi-signature transaction ----------");
    Ok(call_hash)
}

pub(crate) async fn get_last_time_point<T: Runtime + Multisig>(
    subxt_client: &Client<T>,
    multisig_account: T::AccountId,
    call_hash: [u8; 32],
) -> Option<kusama::api::Timepoint<T::BlockNumber>> {
    println!("get time point, waiting...");
    loop {
        let store = kusama::api::MultisigsStore::<T> {
            multisig_account: multisig_account.clone(),
            call_hash: call_hash.clone(),
        };
        if let Some(Some(kusama::api::MultisigData { when, .. })) = subxt_client
            .fetch(&store, None)
            .await
            .map_err(|e| println!("error get_time_point: {:?}", e))
            .ok()
        {
            return Some(when);
        }
        thread::sleep(time::Duration::from_secs(1));
    }
}

pub(crate) async fn get_withdraw_call_hash(
    subxt_client: &Client<HeikoRuntime>,
) -> Result<[u8; 32], Error> {
    println!("[+] Crate last withdraw transaction");
    println!("---------- start create multi-signature transaction ----------");
    // 1.1 construct balance transfer call
    // todo change blances_transfer to withdraw
    let dest = AccountKeyring::Eve.to_account_id().into();
    let inner_call =
        heiko::api::liquid_staking_withdraw_call::<HeikoRuntime>(dest, MIN_WITHDRAW_BALANCE);

    let inner_call_encoded = subxt_client
        .encode(inner_call)
        .map_err(|e| Error::SubxtError(e))?;
    let sudo_call = sudo::SudoCall::<HeikoRuntime> {
        _runtime: PhantomData,
        call: &inner_call_encoded,
    };

    // check if timepoint already exist.
    let call_hash =
        heiko::api::multisig_call_hash(subxt_client, sudo_call.clone()).map_err(|_e| {
            Error::SubxtError(SubError::Other("failed to load get call hash".to_string()))
        })?;

    Ok(call_hash)
}

pub(crate) async fn wait_transfer_finished(
    subxt_client: &Client<HeikoRuntime>,
    account_id: AccountId,
    call_hash: [u8; 32],
) -> Result<(), Error> {
    // todo check if the transaction is in block
    println!("transferring, waiting...");
    thread::sleep(time::Duration::from_secs(6));

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

pub(crate) async fn get_pool_balances(
    subxt_client: Client<HeikoRuntime>,
    pool_account_id: AccountId,
    currency_id: CurrencyId,
) -> Option<u128> {
    let store = heiko::api::AccountsStore::<HeikoRuntime> {
        account: pool_account_id,
        currency_id,
    };
    let r = subxt_client
        .fetch(&store, None)
        .await
        .map_err(|_| println!("failed to fetch tokens.accounts"))
        .ok();

    if let Some(Some(account_info)) = r {
        let balance = account_info.free - account_info.frozen;
        return Some(balance);
    }

    None
}
