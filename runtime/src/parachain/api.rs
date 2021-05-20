use super::error::Error;
use super::runtime::{CurrencyId, ParaRuntime};
use super::vanilla_oracle::FeedValues;
use sp_core::crypto::Pair as TraitPair;
use sp_core::sr25519::Pair;
use sp_keyring::AccountKeyring;
use sp_runtime::FixedU128;
use substrate_subxt::{balances, Client, ClientBuilder, PairSigner, Signer};
// pub async fn run(cmd: &Parameters) -> Result<(), Error> {
//     let subxt_client = ClientBuilder::<ParaRuntime>::new()
//         // .set_url(&cmd.ws_server)
//         .set_url("ws://127.0.0.1:9844")
//         .skip_type_sizes_check()
//         .build()
//         .await
//         .map_err(|e| {
//             println!("subxt_client error: {:?}", e);
//             Error::XErrot
//         })?;
//     let pair = Pair::from_string(&cmd.key_store, None).map_err(|e| {
//         println!("initial pair error: {:?}", e);
//         Error::XErrot
//     })?;
//     let signer = PairSigner::<ParaRuntime, Pair>::new(pair);

//     submit_price(&subxt_client, &signer)
//             .await
//             .map_err(|e| {
//                 dbg!(e);
//                 Error::XErrot
//             })?;
//     Ok(())
// }

// pub async fn submit_price(
//     subxt_client: &Client<ParaRuntime>,
//     signer: &(dyn Signer<ParaRuntime> + Send + Sync),
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let values = vec![(CurrencyId::DOT,FixedU128::from_inner(100u128))];
//     let result = subxt_client.submit(FeedValues { values }, signer).await?;
//     println!("{:?}", result);
//     Ok(())
// }
