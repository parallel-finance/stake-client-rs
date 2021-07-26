// use crate::listener::listen_pool_balance;
use crate::common::primitives::{AccountId, Amount};

// use crate::DB;
// use chrono::Utc;
// use db::model::Withdraw;
// use db::schema::withdraw::dsl::*;
// use diesel::{self, insert_into, update, ExpressionMethods, QueryDsl, RunQueryDsl};

use core::marker::PhantomData;

use runtime::error::Error;
use runtime::heiko::{self, runtime::HeikoRuntime};
use runtime::kusama::{self};
use runtime::pallets::multisig::Multisig;

use std::{thread, time};
use substrate_subxt::{sudo, Client, Signer};
use substrate_subxt::{Error as SubError, Runtime};

/// The first wallet to call withdraw. No need use 'TimePoint' and call 'approve_as_multi'.
pub(crate) async fn do_first_withdraw(
    others: Vec<AccountId>,
    subxt_client: &Client<HeikoRuntime>,
    signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
    multi_account_id: AccountId,
    amount: Amount,
    threshold: u16,
) -> Result<[u8; 32], Error> {
    println!("[+] Create first withdraw transaction");

    println!("---------- start create multi-signature transaction ----------");
    // 1.1 construct balance transfer call
    // let dest = AccountKeyring::Eve.to_account_id().into();
    let inner_call =
        heiko::api::liquid_staking_withdraw_call::<HeikoRuntime>(multi_account_id, amount);
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
    let result = subxt_client.watch(mc, signer).await?;
    println!("multisig_approve_as_multi_call result {:?}", result);

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
    // let dest = AccountKeyring::Eve.to_account_id().into();
    let inner_call =
        heiko::api::liquid_staking_withdraw_call::<HeikoRuntime>(multi_account_id.clone(), amount);

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
    let result = subxt_client.watch(mc, signer).await?;
    println!("multisig_as_multi_call result {:?}", result);
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
    era_index: u32,
    amount: Amount,
    threshold: u16,
) -> Result<[u8; 32], Error> {
    println!("[+] Create first process_pending_unstake transaction");

    println!("---------- start create multi-signature transaction ----------");
    // 1.1 construct balance transfer call
    let inner_call = heiko::api::liquid_staking_process_pending_unstake_call::<HeikoRuntime>(
        agent, owner, era_index, amount,
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
    let result = subxt_client.watch(mc, signer).await?;
    println!("multisig_approve_as_multi_call result {:?}", result);

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
    era_index: u32,
    amount: Amount,
    threshold: u16,
) -> Result<[u8; 32], Error> {
    println!("[+] Crate last process_pending_unstake transaction");
    println!("---------- start create multi-signature transaction ----------");
    // construct process pending unstake call
    let inner_call = heiko::api::liquid_staking_process_pending_unstake_call::<HeikoRuntime>(
        agent, owner, era_index, amount,
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
    let result = subxt_client.watch(mc, signer).await?;
    println!("multisig_as_multi_call result {:?}", result);
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
    let result = subxt_client.watch(mc, signer).await?;
    println!(
        "[finish_processed_unstake] multisig_approve_as_multi_call result {:?}",
        result
    );

    // get account_id of multi address
    let call_hash = kusama::api::multisig_call_hash(subxt_client, sudo_call)?;
    println!(
        "[finish_processed_unstake] call hash {:?}",
        format!("0x{}", hex::encode(call_hash))
    );
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
    let result = subxt_client.watch(mc, signer).await?;
    println!(
        "[finish_processed_unstake] multisig_as_multi_call result {:?}",
        result
    );
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

pub(crate) async fn wait_transfer_finished(
    subxt_client: &Client<HeikoRuntime>,
    account_id: AccountId,
    call_hash: [u8; 32],
) -> Result<(), Error> {
    // todo check if the transaction is in block
    println!("transferring, waiting...");
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
