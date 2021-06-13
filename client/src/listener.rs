use crate::primitives::{AccountId, MIN_POOL_BALANCE};
pub use parallel_primitives::CurrencyId;
use runtime::heiko;
use runtime::heiko::runtime::HeikoRuntime;
use sp_core::crypto::Ss58Codec;
use std::{thread, time};
use substrate_subxt::Client;

/// listen to the balance change of pool
pub(crate) async fn listen_pool_balances(
    subxt_client: Client<HeikoRuntime>,
    ws_server: &str,
    pool_addr: &str,
    currency_id: CurrencyId,
) -> Result<u128, String> {
    let account_id = AccountId::from_string(pool_addr).map_err(|_| "invalid pool address")?;
    let store = heiko::api::AccountsStore::<HeikoRuntime> {
        account: account_id,
        currency_id,
    };
    loop {
        let r = subxt_client
            .fetch(&store, None)
            .await
            .map_err(|_| "failed to fetch tokens.accounts")?;
        let times = time::Duration::from_secs(5);
        thread::sleep(times);

        if let Some(account_info) = r {
            let balance = account_info.free - account_info.frozen;
            if balance >= MIN_POOL_BALANCE {
                println!(
                    "[+] Pool's amount is {:?}， need to withdraw",
                    MIN_POOL_BALANCE
                );
                return Ok(MIN_POOL_BALANCE);
            }
        }
    }
}
