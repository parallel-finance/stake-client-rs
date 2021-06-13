use crate::primitives::{AccountId, MIN_POOL_BALANCE};
pub use parallel_primitives::CurrencyId;
use runtime::heiko;
use runtime::heiko::runtime::HeikoRuntime;
use std::{thread, time};
use substrate_subxt::Client;

/// listen to the balance change of pool
pub(crate) async fn listen_pool_balances(
    subxt_client: Client<HeikoRuntime>,
    pool_account_id: AccountId,
    currency_id: CurrencyId,
) -> Result<u128, String> {
    let store = heiko::api::AccountsStore::<HeikoRuntime> {
        account: pool_account_id,
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
