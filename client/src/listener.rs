use crate::primitives::{AccountId, MIN_POOL_BALANCE};
use orml_tokens::AccountData;
pub use parallel_primitives::CurrencyId;
use runtime::error::Error;
use runtime::heiko;
use runtime::heiko::api::{AccountStore, AccountsStore, TotalStakingAssetStore};
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
) -> Result<u128, String> {
    let account_id = AccountId::from_string(pool_addr).map_err(|_| "invalid pool address")?;
    let store = heiko::api::AccountStore::<HeikoRuntime> {
        account: account_id,
    };
    let mut amount: u128 = 0;
    loop {
        let r = subxt_client
            .fetch(&store, None)
            .await
            .map_err(|_| "failed to fetch system.account")?;
        let times = time::Duration::from_secs(5);
        thread::sleep(times);

        if let Some(account_info) = r {
            let balance = account_info.data.free - account_info.data.misc_frozen;
            if balance >= MIN_POOL_BALANCE {
                println!(
                    "[+] Pool's {:?} amount is {:?}， need to withdraw",
                    currency_id, account_info.data.free
                );
                amount = balance;
                break;
            }
        }
    }

    Ok(amount)
}
