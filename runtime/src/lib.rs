#[macro_use]
extern crate substrate_subxt_proc_macro;

mod relaychain;
mod parachain;
mod pallets;
mod error;
use async_std::task;

//TODO 定义一个结构体接收参数，不依赖command
#[derive(Debug)]
pub struct Parameters {
    pub ws_server: String,
    pub key_store: String,
}


pub fn run(cmd: &Parameters) -> Result<(),error::Error> {
    println!("cmd:{:?}", cmd);

    let _ = task::block_on(relaychain::api::run(cmd))?;
    let _ = task::block_on(parachain::api::run(cmd))?;

    Ok(())
}

