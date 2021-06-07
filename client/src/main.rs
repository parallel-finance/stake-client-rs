mod command;
mod config;
mod crypto;
mod error;
mod keystore;
mod kusama;
mod listener;
mod pkcs8;
mod primitives;
mod tasks;
// mod test;
mod wallet;

use config::Config;
use crypto::*;
use db::executor::DbExecutor;
use lazy_static::lazy_static;
use log::{info, warn};
use primitives::AccountId;
use std::fs;
use tasks::start_withdraw_task;
use wallet::*;
lazy_static! {
    pub static ref CFG: Config =
        { Config::from_file("Config.toml").unwrap_or_else(|_| std::process::exit(1)) };
    pub static ref DB: DbExecutor = {
        let url = CFG.get_postgres_url();
        DbExecutor::new(&url).unwrap_or_else(|err| {
            println!("exit err:{:?}", err);
            std::process::exit(1)
        })
    };
}

#[async_std::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let mut app = command::get_app();
    let matches = app.clone().get_matches();

    match matches.subcommand() {
        ("start", Some(matches)) => {
            // let cmd = command::Cmd::from_args();
            // let p = test::Parameters {
            //     ws_server: cmd.ws_server,
            //     key_store: cmd.key_store,
            // };
            //
            // // let conn = DB.get_connection().unwrap();
            // // withdraw_tx.load::<WithdrawTx>(&conn).unwrap();
            //
            // let _r = test::run(&p);
            println!("start client ...");
            let file = matches.value_of("file").unwrap();
            let ws_server = matches.value_of("ws_server").unwrap();
            let pool_addr = matches.value_of("pool_addr").unwrap();

            // get keystore
            let keystore = get_keystore(file.to_string()).unwrap();
            println!("{:?}", keystore);

            // get pair
            let password = rpassword::read_password_from_tty(Some("Input password:")).ok();
            let pair = keystore.into_pair::<Sr25519>(password).unwrap();

            // get other signatories
            let other_signatories = keystore.get_other_signatories().unwrap();
            let r = start_withdraw_task(pair, other_signatories, ws_server, pool_addr).await;
            println!("start_withdraw_task finished:{:?}", r);
        }

        ("startrelay", Some(matches)) => {
            info!("start relaychain client ...");
            let file = matches.value_of("file").unwrap();
            let relay_ws_server = matches
                .value_of("relay_ws_server")
                .unwrap_or_else(|| "ws://127.0.0.1:19944");
            let para_ws_server = matches
                .value_of("para_ws_server")
                .unwrap_or_else(|| "ws://127.0.0.1:19844");
            let relay_pool_addr = matches
                .value_of("relay_pool_addr")
                .unwrap_or_else(|| "5DjYJStmdZ2rcqXbXGX7TW85JsrW6uG4y9MUcLq2BoPMpRA7");
            let first = matches.value_of("first").unwrap();

            // get keystore
            let keystore = get_keystore(file.to_string()).unwrap();
            info!("{:?}", keystore);

            // get pair
            let password = rpassword::read_password_from_tty(Some("Input password:")).ok();
            let pair = keystore.into_pair::<Sr25519>(password).unwrap();

            // get other signatories
            let other_signatories = keystore.get_other_signatories().unwrap();

            let temporary_cmd = kusama::TemporaryCmd {
                relay_ws_server: relay_ws_server.to_string(),
                para_ws_server: para_ws_server.to_string(),
                relay_key_pair: pair.clone(),
                para_key_pair: pair.clone(),
                relay_pool_addr: relay_pool_addr.to_string(),
                para_pool_addr: "NULL".to_string(),
                relay_multi_other_signatories: other_signatories.clone(),
                para_multi_other_signatories: other_signatories.clone(),
                first: first == "true",
            };
            kusama::run(&temporary_cmd).await?;
            info!("relaychain client finished");
        }

        ("getaddress", Some(_)) => {
            println!("todo addr");
        }

        ("getmultiaddress", Some(_)) => {
            println!("todo multi addr");
        }

        ("show", Some(_)) => {
            println!("todo show");
        }

        ("create", Some(matches)) => {
            let name = matches.value_of("name").unwrap();
            let threshold = matches.value_of("threshold").unwrap();
            let others = matches.value_of("others").unwrap();
            let split = others.split(",");
            let others_split: Vec<&str> = split.collect();
            let mut other_addresses = vec![];
            for a in others_split.iter() {
                let _check = AccountId::from_ss58check(a).map_err(|_err| "Invalid address")?;
                other_addresses.push(a.to_string());
            }

            // create multi signature keystore
            let mut seed: String = "".to_string();
            if let Some(s) = rpassword::read_password_from_tty(Some("Input seed:")).ok() {
                seed = s;
            } else {
                println!("invalid seed")
            }
            let password = rpassword::read_password_from_tty(Some("Set password: ")).ok();
            let keystore = create_keystore(
                password,
                threshold.to_string().parse().unwrap(),
                seed,
                other_addresses,
            )?;

            // create keystore file
            let file_name = format!("{}.json", name);
            if let Err(e) = fs::write(file_name.clone(), keystore.to_json()) {
                println!("failed to write to file: {:?}", e);
            } else {
                println!("keystore file created: {}\n{:?}", file_name, keystore);
            }
        }

        _ => {
            app.print_help().unwrap();
            println!();
        }
    }
    Ok(())
}
