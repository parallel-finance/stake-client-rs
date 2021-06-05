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
) -> Result<u128, String> {
    let account_id = AccountId::from_string(pool_addr).map_err(|_| "invalid pool address")?;
    let store = heiko::api::AccountStore::<HeikoRuntime> {
        account: account_id,
    };
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
                    "[+] Pool's amount is {:?}， need to withdraw",
                    account_info.data.free
                );
                return Ok(balance);
            }
        }
    }
}
