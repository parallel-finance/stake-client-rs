use sp_core::crypto::Pair as TraitPair;
use sp_core::sr25519::Pair;
use substrate_subxt::{Client, ClientBuilder, PairSigner, Signer,balances};
use super::para_runtime::ParaRuntime;
use crate::error::Error;
use crate::command::Cmd;
use sp_keyring::AccountKeyring;
use super::para_runtime::FeedValues;
use sp_runtime::FixedU128;
pub async fn run(cmd: &Cmd) -> Result<(), Error> {
    let subxt_client = ClientBuilder::<ParaRuntime>::new()
        // .set_url(&cmd.ws_server)
        .set_url("ws://127.0.0.1:9844")
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
    let signer = PairSigner::<ParaRuntime, Pair>::new(pair);

    submit_price(&subxt_client, &signer)
            .await
            .map_err(|e| {
                dbg!(e);
                Error::XErrot
            })?;
    Ok(())
}

pub async fn submit_price(
    subxt_client: &Client<ParaRuntime>,
    signer: &(dyn Signer<ParaRuntime> + Send + Sync),
) -> Result<(), Box<dyn std::error::Error>> {
    let values = vec![(super::para_runtime::CurrencyId::DOT,FixedU128::from_inner(100u128))];
    let result = subxt_client.submit(FeedValues { values }, signer).await?;
    // let dest = AccountKeyring::Bob.to_account_id().into();
    // let result = subxt_client
    //     .submit(
    //         FeedValues {
    //             to: &dest,
    //             amount: 10_000u128,
    //         },
    //         signer,
    //     )
    //     .await
    //     .unwrap();
    println!("{:?}", result);
    Ok(())
}