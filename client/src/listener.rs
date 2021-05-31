use crate::primitives::{AccountId, CurrencyId, MIN_POOL_BALANCE};
use orml_tokens::AccountData;
use schnorrkel::MINI_SECRET_KEY_LENGTH;
use serde_json::to_value;
use sp_core::crypto::{CryptoType, SecretStringError, Ss58Codec};
use sp_core::{sr25519, sr25519::Signature, DeriveJunction, Pair};
use std::{thread, time};
use substrate_api_client::rpc::json_req::state_get_storage_by_str;
use substrate_api_client::AccountInfo;
use substrate_api_client::Api;

/// listen to the balance change of pool
pub(crate) async fn listen_pool_balances(
    ws_server: &str,
    pool_addr: &str,
    currency_id: CurrencyId,
) {
    let api = Api::<sr25519::Pair>::new(ws_server.to_string()).unwrap();
    loop {
        let one_second = time::Duration::from_secs(5);
        thread::sleep(one_second);

        let account_id = AccountId::from_string(pool_addr).unwrap();
        let mut key = api
            .metadata
            .storage_double_map_key::<AccountId, CurrencyId, AccountData<u128>>(
                "Tokens",
                "Accounts",
                account_id,
                currency_id,
            )
            .unwrap();
        key.0.push(currency_id as u8);
        let result = api
            .get_storage_by_key_hash::<AccountData<u128>>(key, None)
            .unwrap();

        // let result = api
        //     .get_storage_double_map("Tokens", "Accounts", account_id.clone(), CurrencyId::DOT, None).unwrap();
        let mut account_data: AccountData<u128> = AccountData::default();
        match result {
            Some(v) => {
                account_data = v;
            }
            _ => println!("result is null"),
        }

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

pub(crate) async fn wait_transfer_finished() {
    // todo wait transfer
    let one_second = time::Duration::from_secs(60);
    thread::sleep(one_second);
}
