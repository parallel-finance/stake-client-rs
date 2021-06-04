use super::kusama;
use super::AccountId;
use super::Error;
use super::KusamaRuntime;
use super::MIN_POOL_BALANCE;
use log::{error, info, warn};
use sp_keyring::AccountKeyring;
use substrate_subxt::{Client, Signer};
/// The first wallet to call withdraw. No need use 'TimePoint' and call 'approve_as_multi'.
pub(crate) async fn do_first_relay_bond(
    others: Vec<AccountId>,
    subxt_client: &Client<KusamaRuntime>,
    signer: &(dyn Signer<KusamaRuntime> + Send + Sync),
) -> Result<(), Error> {
    info!("do_first_relay_bond");
    // 1.1 construct staking bond call
    // let ctrl = AccountKeyring::Eve.to_account_id().into();
    // let call = kusama::api::staking_bond_call::<KusamaRuntime>(
    //     &ctrl,
    //     MIN_POOL_BALANCE,
    //     staking::RewardDestination::Staked,
    // );
    // let mc = kusama::api::multisig_approve_as_multi_call::<
    //     KusamaRuntime,
    //     staking::BondCall<KusamaRuntime>,
    // >(
    //     subxt_client,
    //     2,
    //     others,
    //     None,
    //     call,
    //     0u64,
    // )?;
    // // 1.2 initial the multisg call
    // //TODO submit and watch
    // let result = subxt_client.submit(mc, signer).await?;
    // println!("multisig_approve_as_multi_call hash {:?}", result);
    Ok(())
}

/// If the wallet is the last one need to get 'TimePoint' and call 'as_multi'.
pub(crate) async fn do_last_relay_bond(
    others: Vec<AccountId>,
    pool_addr: String,
    subxt_client: &Client<KusamaRuntime>,
    signer: &(dyn Signer<KusamaRuntime> + Send + Sync),
) -> Result<(), Error> {
    info!("do_last_relay_bond");
    // let ctrl = AccountKeyring::Eve.to_account_id().into();
    // let call = kusama::api::staking_bond_call::<KusamaRuntime>(
    //     &ctrl,
    //     MIN_POOL_BALANCE,
    //     staking::RewardDestination::Staked,
    // );
    // let public =
    //     sp_core::ed25519::Public::from_str(pool_addr)
    //         .unwrap();

    // let kusama::api::MultisigData { when, .. } = subxt_client.fetch(&store, None).await?.unwrap();
    // println!("multisig timepoint{:?}", when);

    // let call_hash= kusama::api::multisig_call_hash(subxt_client, call).unwrap();

    // let when = get_last_withdraw_time_point(subxt_client, public.into(), call_hash).await?;

    // // 3.1 approve the call and execute it
    // let bob = PairSigner::<RelayRuntime, _>::new(AccountKeyring::Bob.pair());
    // let mc = kusama::api::multisig_as_multi_call::<RelayRuntime, staking::BondCall<RelayRuntime>>(
    //     subxt_client,
    //     2,
    //     others,
    //     Some(Timepoint::new(when.height, when.index)),
    //     call,
    //     false,
    //     1_000_000_000_000,
    // )?;
    // let result = subxt_client.submit(mc, signer).await.unwrap();
    // println!("multisig_as_multi_call hash {:?}", result);
    Ok(())
}

// pub(crate) async fn get_last_withdraw_time_point(
//     subxt_client: &Client<KusamaRuntime>,
//     account_id: AccountId,
//     call_hash: [u8; 32],
// ) -> Result<Timepoint<u32>, Error> {
//     loop {
//         println!("get time point, waiting...");
//         let store = kusama::api::MultisigsStore::<KusamaRuntime> {
//             multisig_account: account_id.clone(),
//             call_hash: call_hash.clone(),
//         };
//         if let Some(kusama::api::MultisigData { when, .. }) =
//             subxt_client.fetch(&store, None).await?
//         {
//             return Ok(when);
//         }
//         thread::sleep(time::Duration::from_secs(1));
//     }
// }
