mod command;
use structopt::StructOpt;
fn main() {
    let cmd = command::Cmd::from_args();
    let p = runtime::Parameters{
        ws_server:cmd.ws_server,
        key_store:cmd.key_store,
    };
    let _r = runtime::run(&p);
    // println!("main r:{:?}", r);
    db::read();
    db::write();
}
