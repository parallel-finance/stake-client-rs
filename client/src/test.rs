use std::convert::TryFrom;

use async_std::task;
use runtime::error::Error;
use runtime::heiko::{self, runtime::HeikoRuntime};
use runtime::kusama::{self, runtime::KusamaRuntime as RelayRuntime};
use runtime::pallets::multisig::{ApproveAsMultiCall, AsMultiCall, Multisig, Timepoint};
use sp_core::crypto::Pair as TraitPair;
use sp_core::sr25519::Pair;
use sp_keyring::AccountKeyring;
use std::str::FromStr;
use std::time::Duration;
use substrate_subxt::{
    balances, staking, sudo, Client, ClientBuilder, PairSigner, Runtime, Signer,
};
use substrate_subxt::{sp_runtime::traits::IdentifyAccount, Encoded};
///SHOULD BE DELETE!!!

//TODO 定义一个结构体接收参数，不依赖command
#[derive(Debug)]
pub struct Parameters {
    pub ws_server: String,
    pub key_store: String,
}

pub fn run(cmd: &Parameters) -> Result<(), Error> {
    println!("cmd:{:?}", cmd);

    // let _ = task::block_on(run_relaychain(cmd))?;
    let _ = task::block_on(run_parachain(cmd))?;

    Ok(())
}

pub async fn run_relaychain(cmd: &Parameters) -> Result<(), Error> {
    let subxt_client = ClientBuilder::<RelayRuntime>::new()
        .set_url(&cmd.ws_server)
        .register_type_size::<([u8; 20])>("EthereumAddress")
        .build()
        .await
        .map_err(|e| {
            println!("subxt_client error: {:?}", e);
            Error::SubxtError(e)
        })?;
    let pair = Pair::from_string(&cmd.key_store, None).unwrap();
    let signer = PairSigner::<RelayRuntime, Pair>::new(pair);
    test_multisig_balances_transfer(&subxt_client, &signer).await?;
    test_multisig_staking_bond(&subxt_client, &signer).await?;
    Ok(())
}

pub async fn test_multisig_balances_transfer(
    subxt_client: &Client<RelayRuntime>,
    signer: &(dyn Signer<RelayRuntime> + Send + Sync),
) -> Result<(), Error> {
    println!("----------start test_multisig_balances_transfer----------");
    // 1.1 construct balance transfer call
    let dest = AccountKeyring::Eve.to_account_id().into();
    let call = kusama::api::balances_transfer_call::<RelayRuntime>(&dest, 3_000_000_000_000u128);
    let mc = kusama::api::multisig_approve_as_multi_call::<
        RelayRuntime,
        balances::TransferCall<RelayRuntime>,
    >(
        subxt_client,
        2,
        vec![
            AccountKeyring::Bob.to_account_id(),
            AccountKeyring::Charlie.to_account_id(),
        ],
        None,
        call.clone(),
        0u64,
    )?;
    // 1.2 initial the multisg call
    let result = subxt_client.submit(mc, signer).await.unwrap();
    println!("multisig_approve_as_multi_call hash {:?}", result);

    // 2.1 waiting block produce
    task::sleep(Duration::from_millis(6000)).await;

    // 2.2 get timepoint
    let public =
        sp_core::ed25519::Public::from_str("12fqSn9qVLJL4NY7Uua7bexEAVr9oCpD3e5xmdpNjtQszzBt")
            .unwrap();
    let store = kusama::api::MultisigsStore::<RelayRuntime> {
        multisig_account: public.into(),
        call_hash: kusama::api::multisig_call_hash(subxt_client, call.clone()).unwrap(),
    };

    let kusama::api::MultisigData { when, .. } = subxt_client.fetch(&store, None).await?.unwrap();
    println!("multisig timepoint{:?}", when);

    // 3.1 approve the call and execute it
    let bob = PairSigner::<RelayRuntime, _>::new(AccountKeyring::Bob.pair());
    let mc =
        kusama::api::multisig_as_multi_call::<RelayRuntime, balances::TransferCall<RelayRuntime>>(
            subxt_client,
            2,
            vec![
                AccountKeyring::Charlie.to_account_id(),
                AccountKeyring::Alice.to_account_id(),
            ],
            Some(Timepoint::new(when.height, when.index)),
            call,
            false,
            1_000_000_000_000,
        )?;
    let result = subxt_client.submit(mc, &bob).await.unwrap();
    println!("multisig_as_multi_call hash {:?}", result);
    Ok(())
}

pub async fn test_multisig_staking_bond(
    subxt_client: &Client<RelayRuntime>,
    signer: &(dyn Signer<RelayRuntime> + Send + Sync),
) -> Result<(), Error> {
    println!("----------start test_multisig_staking_bond----------");
    // 1.1 construct staking bond call
    let dest = AccountKeyring::Eve.to_account_id().into();
    let call = kusama::api::staking_bond_call::<RelayRuntime>(
        &dest,
        15_000_000_000_000u128,
        staking::RewardDestination::Staked,
    );
    let mc = kusama::api::multisig_approve_as_multi_call::<
        RelayRuntime,
        staking::BondCall<RelayRuntime>,
    >(
        subxt_client,
        2,
        vec![
            AccountKeyring::Bob.to_account_id(),
            AccountKeyring::Charlie.to_account_id(),
        ],
        None,
        call,
        0u64,
    )?;
    // 1.2 initial the multisg call
    let result = subxt_client.submit(mc, signer).await.unwrap();
    println!("multisig_approve_as_multi_call hash {:?}", result);

    // 2.1 waiting block produce
    task::sleep(Duration::from_millis(6000)).await;

    // 2.2 get timepoint
    let call = kusama::api::staking_bond_call::<RelayRuntime>(
        &dest,
        15_000_000_000_000u128,
        staking::RewardDestination::Staked,
    );
    let public =
        sp_core::ed25519::Public::from_str("12fqSn9qVLJL4NY7Uua7bexEAVr9oCpD3e5xmdpNjtQszzBt")
            .unwrap();
    let store = kusama::api::MultisigsStore::<RelayRuntime> {
        multisig_account: public.into(),
        call_hash: kusama::api::multisig_call_hash(subxt_client, call).unwrap(),
    };

    let kusama::api::MultisigData { when, .. } = subxt_client.fetch(&store, None).await?.unwrap();
    println!("multisig timepoint{:?}", when);

    // 3.1 approve the call and execute it
    let call = kusama::api::staking_bond_call::<RelayRuntime>(
        &dest,
        15_000_000_000_000u128,
        staking::RewardDestination::Staked,
    );
    let bob = PairSigner::<RelayRuntime, _>::new(AccountKeyring::Bob.pair());
    let mc = kusama::api::multisig_as_multi_call::<RelayRuntime, staking::BondCall<RelayRuntime>>(
        subxt_client,
        2,
        vec![
            AccountKeyring::Charlie.to_account_id(),
            AccountKeyring::Alice.to_account_id(),
        ],
        Some(Timepoint::new(when.height, when.index)),
        call,
        false,
        1_000_000_000_000,
    )?;
    let result = subxt_client.submit(mc, &bob).await.unwrap();
    println!("multisig_as_multi_call hash {:?}", result);
    Ok(())
}

pub async fn run_parachain(cmd: &Parameters) -> Result<(), Error> {
    let subxt_client = ClientBuilder::<HeikoRuntime>::new()
        .set_url(&cmd.ws_server)
        .skip_type_sizes_check()
        .build()
        .await
        .map_err(|e| {
            println!("subxt_client error: {:?}", e);
            Error::SubxtError(e)
        })?;
    let pair = Pair::from_string(&cmd.key_store, None).unwrap();
    let signer = PairSigner::<HeikoRuntime, Pair>::new(pair);
    test_staking_pallet(&subxt_client, &signer).await?;
    Ok(())
}

pub async fn test_staking_pallet(
    subxt_client: &Client<HeikoRuntime>,
    signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
) -> Result<(), Error> {
    // 1 test stake
    let call = heiko::api::staking_stake_call::<HeikoRuntime>(8_000_000_000_000u128);
    let result = subxt_client.submit(call, signer).await.unwrap();
    println!("test_heiko_staking_stake hash {:?}", result);

    // 2 test withdraw
    task::sleep(Duration::from_millis(6000)).await;
    let dest = AccountKeyring::Eve.to_account_id().into();
    let call = heiko::api::staking_withdraw_call::<HeikoRuntime>(dest, 500u128);

    let call_encoded = subxt_client
        .encode(call)
        .map_err(|e| Error::SubxtError(e))?;
    let sc = sudo::SudoCall::<HeikoRuntime> {
        _runtime: core::marker::PhantomData,
        call: &call_encoded,
    };
    let result = subxt_client.submit(sc, signer).await.unwrap();
    println!("test_heiko_staking_withdraw hash {:?}", result);

    // 3 test record rewards
    task::sleep(Duration::from_millis(6000)).await;
    let dest = AccountKeyring::Eve.to_account_id().into();
    let call = heiko::api::staking_record_rewards_call::<HeikoRuntime>(dest, 111_000_000_000u128);

    let call_encoded = subxt_client
        .encode(call)
        .map_err(|e| Error::SubxtError(e))?;
    let sc = sudo::SudoCall::<HeikoRuntime> {
        _runtime: core::marker::PhantomData,
        call: &call_encoded,
    };
    let result = subxt_client.submit(sc, signer).await.unwrap();
    println!("test_heiko_staking_record_rewards hash {:?}", result);
    Ok(())
}
