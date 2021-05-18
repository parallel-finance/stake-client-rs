#[macro_use]
extern crate substrate_subxt_proc_macro;

mod para_api;
mod para_runtime;
mod relay_api;
mod relay_runtime;
mod error;
use crate::error::Error;
use async_std::task;

//TODO 定义一个结构体接收参数，不依赖command
#[derive(Debug)]
pub struct Parameters {
    pub ws_server: String,
    pub key_store: String,
}


pub fn run(cmd: &Parameters) -> Result<(),Error> {
    println!("cmd:{:?}", cmd);

    let _ = task::block_on(relay_api::run(cmd))?;
    let _ = task::block_on(para_api::run(cmd))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
