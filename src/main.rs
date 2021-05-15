#[macro_use]
extern crate substrate_subxt_proc_macro;
mod command;
mod client;
mod error;
use structopt::StructOpt;
fn main() {
    let cmd = command::Cmd::from_args();
    let _r = client::run(&cmd);
    // println!("main r:{:?}", r);
}
