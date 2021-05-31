mod command;
mod config;
mod crypto;
mod error;
mod keystore;
mod listener;
mod pkcs8;
mod primitives;
mod tasks;
mod test;
mod wallet;

use crate::keystore::Keystore;
use async_std::task;
use config::Config;
use crypto::*;
use db::executor::DbExecutor;
use db::model::WithdrawTx;
use db::schema::withdraw_tx::dsl::*;
use lazy_static::lazy_static;
use listener::{listen_pool_balances, wait_transfer_finished};
use primitives::AccountId;
use primitives::CurrencyId;
use sp_core::crypto::Pair as TraitPair;
use sp_core::sr25519::Pair;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;
use tasks::{do_first_withdraw, do_last_withdraw, do_middle_withdraw};
use wallet::*;

const DEFAULT_WS_SERVER: &str = "ws://127.0.0.1:9944";
const DEFAULT_POOL_ADDR: &str = "5DjYJStmdZ2rcqXbXGX7TW85JsrW6uG4y9MUcLq2BoPMpRA7";

lazy_static! {
    pub static ref CFG: Config =
        { Config::from_file("Config.toml").unwrap_or_else(|_| std::process::exit(1)) };
    pub static ref DB: DbExecutor = {
        let url = CFG.get_postgres_url();
        DbExecutor::new(&url).unwrap_or_else(|_| std::process::exit(1))
    };
}

/// start withdraw task, ws_server: ws://127.0.0.1:9944
pub(crate) async fn start_withdraw_task(
    keystore: Keystore,
    pair: Pair,
    mut ws_server: &str,
    mut pool_addr: &str,
    first: bool, // temp use
) {
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

    if ws_server == "" {
        ws_server = DEFAULT_WS_SERVER;
    }
    if pool_addr == "" {
        pool_addr = DEFAULT_POOL_ADDR;
    }
    loop {
        // todo complete me
        println!("start listen_pool_balances");
        task::block_on(listen_pool_balances(
            ws_server.clone(),
            pool_addr.clone(),
            CurrencyId::KSM,
        ));

        // todo get state from db
        // let conn = DB.get_connection().unwrap();

        // todo do withdraw, first? or second?
        if first {
            task::block_on(do_first_withdraw(
                keystore.clone(),
                pair.clone(),
                ws_server.clone(),
                pool_addr.clone(),
            ));
            task::block_on(wait_transfer_finished())
        } else {
            task::block_on(do_last_withdraw());
            task::block_on(wait_transfer_finished())
        }
        //task::block_on(do_middle_withdraw());
    }
}

#[async_std::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut app = command::get_app();
    let matches = app.clone().get_matches();

    match matches.subcommand() {
        ("start", Some(matches)) => {
            println!("start client ...");
            let file = matches.value_of("file").unwrap();
            let ws_server = matches.value_of("ws_server").unwrap();
            let pool_addr = matches.value_of("pool_addr").unwrap();
            let first = matches.value_of("first").unwrap();

            let keystore = get_keystore(file.to_string()).unwrap();
            println!("{:?}", keystore);

            let password = rpassword::read_password_from_tty(Some("Input password:")).ok();
            let signer = keystore.into_pair::<Sr25519>(password).unwrap();
            task::block_on(start_withdraw_task(
                keystore,
                signer,
                ws_server,
                pool_addr,
                first == "true",
            ));
        }

        ("getaddress", Some(_)) => {
            println!("todo addr");
        }

        ("getmultiaddress", Some(_)) => {
            println!("todo multi addr");
        }

        ("show", Some(matches)) => {
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
