mod config;
mod error;
mod command;

use structopt::StructOpt;
use lazy_static::lazy_static;

use diesel::{
    self, query_dsl::BelongingToDsl, result::Error, BoolExpressionMethods, Connection,
    ExpressionMethods, QueryDsl, RunQueryDsl,
};
use config::Config;
use db::executor::DbExecutor;
use db::model::WithdrawTx;
use db::schema::withdraw_tx::dsl::*;

lazy_static! {
    pub static ref CFG: Config = {
        Config::from_file("Config.toml").unwrap_or_else(|_| std::process::exit(1))
    };
    pub static ref DB: DbExecutor = {
        let url = CFG.get_postgres_url();
        DbExecutor::new(&url).unwrap_or_else(|_| std::process::exit(1))
    };
}

fn main() {
    let cmd = command::Cmd::from_args();
    let p = runtime::Parameters {
        ws_server: cmd.ws_server,
        key_store: cmd.key_store,
    };

    let conn = DB.get_connection().unwrap();
    withdraw_tx.load::<WithdrawTx>(&conn).unwrap();

    let _r = runtime::run(&p);
}
