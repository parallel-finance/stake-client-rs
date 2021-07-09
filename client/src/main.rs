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
mod tasks_para;
// mod test;
mod wallet;

use crate::command::{CreateCmd, StakeClient, StartParaCmd, StartRelayCmd};

use config::Config;
use crypto::*;
use db::executor::DbExecutor;
// use frame_support::PalletId;
use lazy_static::lazy_static;
use log::info;
use parallel_primitives::CurrencyId;
use primitives::AccountId;
use std::fs;
use structopt::StructOpt;
use wallet::*;

lazy_static! {
    pub static ref CFG: Config =
        Config::from_file("Config.toml").unwrap_or_else(|_| std::process::exit(1));
    pub static ref DB: DbExecutor = {
        let url = CFG.get_postgres_url();
        DbExecutor::new(&url).unwrap_or_else(|err| {
            println!("exit err:{:?}", err);
            std::process::exit(1)
        })
    };
}

#[async_std::main]
async fn main() {
    env_logger::init();
    run().await;
}

async fn run() {
    match StakeClient::from_args() {
        StakeClient::Create(cmd) => cmd.run(),
        StakeClient::StartPara(cmd) => cmd.run().await,
        StakeClient::StartRelay(cmd) => cmd.run().await,
    }
}

impl CreateCmd {
    /// Run the command
    pub fn run(&self) {
        let mut other_addresses = vec![];
        for a in self.other_signatories.iter() {
            println!("a:{:?}", a);
            let _check = AccountId::from_ss58check(a).unwrap();
            other_addresses.push(a);
        }

        // create multi signature keystore
        if let Some(seed) = rpassword::read_password_from_tty(Some("Type seed:")).ok() {
            let password = rpassword::read_password_from_tty(Some("Type password:")).ok();
            match create_keystore(
                password,
                self.threshold.clone(),
                seed,
                self.other_signatories.clone(),
            ) {
                Ok(keystore) => {
                    // create keystore file
                    let file_name = format!("{}.json", self.name);
                    if let Err(e) = fs::write(file_name.clone(), keystore.to_json()) {
                        println!("failed to write to file: {:?}", e);
                    } else {
                        println!("keystore file created: {}\n{:?}", file_name, keystore);
                    }
                }
                Err(e) => {
                    println!("create keystore error:{:?}", e);
                }
            }
        } else {
            println!("invalid seed")
        }
    }
}

impl StartParaCmd {
    pub async fn run(&self) {
        // let account_id: AccountId = PalletId(*b"par/stak").into_account();
        // println!("palletId address:{}", account_id.to_string());

        // get pair
        let password: Option<String>;
        match &self.password {
            Some(p) => password = Some(p.to_string()),
            None => password = rpassword::read_password_from_tty(Some("Type password:")).ok(),
        }

        // get keystore
        let keystore = get_keystore(self.key_store.to_string()).unwrap();
        println!("{:?}", keystore);

        let pair = keystore.into_pair::<Sr25519>(password).unwrap();

        // get other signatories
        let other_signatories = keystore.get_other_signatories().unwrap();
        let r = tasks_para::run(
            keystore.threshold,
            pair,
            other_signatories,
            &self.para_ws_server,
            &self.relay_ws_server,
            &self.para_pool_addr,
            &keystore.multi_address,
            CurrencyId::KSM,
            self.first,
        )
        .await;
        println!("para chain client finished:{:?}", r);
    }
}

impl StartRelayCmd {
    pub async fn run(&self) {
        // get pair
        let password: Option<String>;
        match &self.password {
            Some(p) => password = Some(p.to_string()),
            None => password = rpassword::read_password_from_tty(Some("Type password:")).ok(),
        }

        // get keystore
        let keystore = get_keystore(self.key_store.to_string()).unwrap();
        info!("{:?}", keystore);

        let pair = keystore.into_pair::<Sr25519>(password).unwrap();

        // get other signatories
        let other_signatories = keystore.get_other_signatories().unwrap();

        let temporary_cmd = kusama::TemporaryCmd {
            relay_ws_server: self.relay_ws_server.clone(),
            para_ws_server: self.para_ws_server.clone(),
            relay_key_pair: pair.clone(),
            para_key_pair: pair.clone(),
            relay_pool_addr: self.relay_pool_addr.clone(),
            para_pool_addr: "NULL".to_string(),
            relay_multi_other_signatories: other_signatories.clone(),
            para_multi_other_signatories: other_signatories.clone(),
            first: self.first,
        };
        let r = kusama::run(&temporary_cmd).await;
        info!("relaychain client finished {:?}", r);
    }
}
