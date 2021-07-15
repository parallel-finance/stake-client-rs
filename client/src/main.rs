mod command;
mod common;
mod keystore;
mod kusama;
mod parallel;
mod postgres;
// mod test;

use crate::command::StakeClient;
use structopt::StructOpt;

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
