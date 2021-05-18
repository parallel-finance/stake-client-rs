use sp_core::crypto::Pair as TraitPair;
use sp_core::sr25519::Pair;
use substrate_subxt::{Client, ClientBuilder, PairSigner, Signer,balances};
use super::runtime::RelayRuntime;
use super::error::Error;
use super::Parameters;
use sp_keyring::AccountKeyring;
pub async fn run(cmd: &Parameters) -> Result<(), Error> {
    let subxt_client = ClientBuilder::<RelayRuntime>::new()
        .set_url(&cmd.ws_server)
        .skip_type_sizes_check()
        .build()
        .await
        .map_err(|e| {
            println!("subxt_client error: {:?}", e);
            Error::XErrot
        })?;
    let pair = Pair::from_string(&cmd.key_store, None).map_err(|e| {
        println!("initial pair error: {:?}", e);
        Error::XErrot
    })?;
    let signer = PairSigner::<RelayRuntime, Pair>::new(pair);

    submit_price(&subxt_client, &signer)
            .await
            .map_err(|e| {
                dbg!(e);
                Error::XErrot
            })?;
    Ok(())
}

pub async fn submit_price(
    subxt_client: &Client<RelayRuntime>,
    signer: &(dyn Signer<RelayRuntime> + Send + Sync),
) -> Result<(), Box<dyn std::error::Error>> {
    // let result = subxt_client.submit(FeedValues { values }, signer).await?;
    let dest = AccountKeyring::Bob.to_account_id().into();
    let result = subxt_client
        .submit(
            balances::TransferCall {
                to: &dest,
                amount: 10_000u128,
            },
            signer,
        )
        .await
        .unwrap();
    println!("{:?}", result);
    Ok(())
}