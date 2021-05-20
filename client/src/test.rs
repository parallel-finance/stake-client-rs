use async_std::task;
use runtime::error::Error;
use runtime::pallets::multisig::{ApproveAsMultiCall, AsMultiCall, Multisig, Timepoint};
use runtime::parachain;
use runtime::relaychain::{self, runtime::RelayRuntime};
use sp_core::crypto::Pair as TraitPair;
use sp_core::sr25519::Pair;
use sp_keyring::AccountKeyring;
use substrate_subxt::Encoded;
use substrate_subxt::{balances, Client, ClientBuilder, PairSigner, Signer};

///SHOULD BE DELETE!!!

//TODO 定义一个结构体接收参数，不依赖command
#[derive(Debug)]
pub struct Parameters {
    pub ws_server: String,
    pub key_store: String,
}

pub fn run(cmd: &Parameters) -> Result<(), Error> {
    println!("cmd:{:?}", cmd);

    let _ = task::block_on(run1(cmd))?;
    // let _ = task::block_on(parachain::api::run(cmd))?;

    Ok(())
}

pub async fn run1(cmd: &Parameters) -> Result<(), Error> {
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
    submit(&subxt_client, &signer).await?;
    Ok(())
}

pub async fn submit(
    subxt_client: &Client<RelayRuntime>,
    signer: &(dyn Signer<RelayRuntime> + Send + Sync),
) -> Result<(), Error> {
    let dest = AccountKeyring::Eve.to_account_id().into();
    let call =
        relaychain::api::balances_transfer_call::<RelayRuntime>(&dest, 2_000_000_000_000u128);
    // let mc = relaychain::api::multisig_approve_as_multi_call::<RelayRuntime,balances::TransferCall<RelayRuntime>>(
    //     subxt_client,
    //     2,
    //     vec![AccountKeyring::Bob.to_account_id(),AccountKeyring::Charlie.to_account_id()],
    //     None,
    //     call,
    //     0u64
    // )?;
    // let result = subxt_client.submit(mc,signer,).await.unwrap();

    //TODO 根据交易hash获取timepoint，如果成功返回，插入到db中

    let bob = PairSigner::<RelayRuntime, _>::new(AccountKeyring::Bob.pair());
    let mc = relaychain::api::multisig_as_multi_call::<
        RelayRuntime,
        balances::TransferCall<RelayRuntime>,
    >(
        subxt_client,
        2,
        vec![
            AccountKeyring::Charlie.to_account_id(),
            AccountKeyring::Alice.to_account_id(),
        ],
        Some(Timepoint::new(37076, 1)),
        call,
        false,
        1_000_000_000_000,
    )?;
    let result = subxt_client.submit(mc, &bob).await.unwrap();

    println!("{:?}", result);
    Ok(())
}
