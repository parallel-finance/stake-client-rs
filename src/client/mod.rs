mod para_api;
mod para_runtime;
mod relay_api;
mod relay_runtime;
use crate::command::Cmd;
use crate::error::Error;
use async_std::task;
pub fn run(cmd: &Cmd) -> Result<(),Error> {
    println!("cmd:{:?}", cmd);
    let _ = task::block_on(relay_api::run(cmd))?;
    let _ = task::block_on(para_api::run(cmd))?;

    Ok(())
}

