use crate::keystore::Keystore;
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

/// The first wallet to call withdraw. No need use 'TimePoint' and call 'approve_as_multi'.
pub(crate) async fn do_first_withdraw(
    keystore: Keystore,
    pair: Pair,
    ws_server: &str,
    pool_addr: &str,
) -> Result<(), Error> {
    println!("do_first_withdraw");

    let subxt_client = ClientBuilder::<HeikoRuntime>::new()
        .set_url(ws_server)
        .skip_type_sizes_check()
        .build()
        .await
        .map_err(|e| {
            println!("subxt_client error: {:?}", e);
            Error::SubxtError(e)
        })?;
    // let pair = Pair::from_string(&cmd.key_store, None).unwrap();
    let signer = PairSigner::<HeikoRuntime, Pair>::new(pair);
    start_multi_transfer(&subxt_client, &signer).await?;
    Ok(())
}

/// If the wallet is the middle one to call withdraw, need to get 'TimePoint' and call 'approve_as_multi'.
pub(crate) async fn do_middle_withdraw() -> Result<(), Error> {
    println!("do_middle_withdraw");
    Ok(())
}

/// If the wallet is the last one need to get 'TimePoint' and call 'as_multi'.
pub(crate) async fn do_last_withdraw() -> Result<(), Error> {
    println!("do_last_withdraw");
    Ok(())
}

async fn start_multi_transfer(
    subxt_client: &Client<HeikoRuntime>,
    signer: &(dyn Signer<HeikoRuntime> + Send + Sync),
) -> Result<(), Error> {
    println!("---------- start create multi-signature transaction ----------");
    // 1.1 construct balance transfer call
    let dest = AccountKeyring::Eve.to_account_id().into();
    let call = heiko::api::balances_transfer_call::<HeikoRuntime>(&dest, 3_000_000_000_000u128);
    let mc = heiko::api::multisig_approve_as_multi_call::<
        HeikoRuntime,
        balances::TransferCall<HeikoRuntime>,
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
    Ok(())
}
