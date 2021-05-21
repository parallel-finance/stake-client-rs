mod command;
mod config;
mod crypto;
mod error;
mod keystore;
mod pkcs8;
mod primitives;
mod test;
mod wallet;

use config::Config;
use db::executor::DbExecutor;
use db::model::WithdrawTx;
use db::schema::withdraw_tx::dsl::*;
use diesel::{
    self, query_dsl::BelongingToDsl, result::Error, BoolExpressionMethods, Connection,
    ExpressionMethods, QueryDsl, RunQueryDsl,
};
use lazy_static::lazy_static;
use structopt::StructOpt;

use crate::crypto::Sr25519;
use crypto::*;
use frame_system::Event;
use keystore::Keystore;
use primitives::AccountId;
use sp_core::crypto::{set_default_ss58_version, Ss58AddressFormat};
use sp_core::hexdisplay::HexDisplay;
use std::fs;
use std::path::{Path, PathBuf};
use wallet::*;

fn default_keystore_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap();
    path.push("./");
    if !path.exists() {
        fs::create_dir_all(path.clone()).expect("Failed to create default data path");
    }
    path
}

fn default_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap();
    path.push(".stakewallet");
    if !path.exists() {
        fs::create_dir_all(path.clone()).expect("Failed to create default data path");
    }
    path
}

lazy_static! {
    pub static ref CFG: Config =
        { Config::from_file("Config.toml").unwrap_or_else(|_| std::process::exit(1)) };
    pub static ref DB: DbExecutor = {
        let url = CFG.get_postgres_url();
        DbExecutor::new(&url).unwrap_or_else(|_| std::process::exit(1))
    };
}

#[async_std::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut app = command::get_app();
    let matches = app.clone().get_matches();

    let data_path = default_path();
    // let store = WalletStore::init(data_path.as_path().to_str());
    // set_default_ss58_version(Ss58AddressFormat::PolkadotAccount);

    match matches.subcommand() {
        ("start", Some(_)) => {
            println!("start client ...");
            let cmd = command::Cmd::from_args();
            let p = test::Parameters {
                ws_server: cmd.ws_server,
                key_store: cmd.key_store,
            };

            // let conn = DB.get_connection().unwrap();
            // withdraw_tx.load::<WithdrawTx>(&conn).unwrap();

            let _r = test::run(&p);
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
            let name = matches.value_of("name").unwrap_or("");
            let threshold = matches.value_of("threshold").unwrap();
            let seed = matches.value_of("seed").unwrap();
            let others = matches.value_of("others").unwrap();
            let mut split = others.split(",");
            let others_split: Vec<&str> = split.collect();
            let mut other_addresses = vec![];
            for a in others_split.iter() {
                let _check = AccountId::from_ss58check(a).map_err(|_err| "Invalid address")?;
                other_addresses.push(a.to_string());
            }

            // create multi signature keystore
            let password = rpassword::read_password_from_tty(Some("Set password: ")).ok();
            let mut keystore = create_keystore(
                password,
                threshold.to_string().parse().unwrap(),
                seed.to_string(),
                other_addresses,
            )?;

            // create keystore file
            let file_name = format!("{}.json", name);
            if let Err(e) = fs::write(file_name.clone(), keystore.to_json()) {
                println!("failed to write to file: {:?}", e);
            } else {
                println!("keystore file created:{}\n{:?}", file_name, keystore);
            }
        }

        _ => {
            app.print_help().unwrap();
            println!();
        }
    }
    Ok(())
}
