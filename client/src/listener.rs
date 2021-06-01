use crate::primitives::{AccountId, MIN_POOL_BALANCE};
use orml_tokens::AccountData;
pub use parallel_primitives::CurrencyId;
use runtime::error::Error;
use runtime::heiko;
use runtime::heiko::api::{AccountsStore, TotalStakingAssetStore};
use runtime::heiko::runtime::HeikoRuntime;
use schnorrkel::MINI_SECRET_KEY_LENGTH;
use serde_json::to_value;
use sp_core::crypto::{CryptoType, SecretStringError, Ss58Codec};
use sp_core::{sr25519, sr25519::Signature, DeriveJunction, Pair};
use std::{thread, time};
use substrate_subxt::{
    balances, staking, sudo, Client, ClientBuilder, PairSigner, Runtime, Signer,
};

/// listen to the balance change of pool
pub(crate) async fn listen_pool_balances(
    subxt_client: Client<HeikoRuntime>,
    ws_server: &str,
    pool_addr: &str,
    currency_id: CurrencyId,
) -> Result<(), Error> {
    let account_id = AccountId::from_string(pool_addr).unwrap();
    let store = heiko::api::AccountsStore::<HeikoRuntime> {
        account: account_id,
        currency_id: currency_id,
    };
    loop {
        let r = subxt_client.fetch(&store, None).await?;

        let times = time::Duration::from_secs(5);
        thread::sleep(times);

        if let Some(account_data) = r {
            println!("[+] Pool's {:?} amount is {:?}", currency_id, account_data);
            if account_data.free >= MIN_POOL_BALANCE {
                println!(
                    "[+] need to withdraw, current amount {:?}: {:?}",
                    currency_id, account_data.free
                );
                break;
            }
        }
    }

    Ok(())
}

pub(crate) async fn wait_transfer_finished() {
    // todo wait transfer
    let one_second = time::Duration::from_secs(60);
    thread::sleep(one_second);
}
