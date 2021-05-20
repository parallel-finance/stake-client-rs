use clap::{App, Arg, SubCommand};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Cmd {
    /// websocket server endpoint
    #[structopt(short, long, default_value = "ws://127.0.0.1:9944")]
    pub ws_server: String,

    #[structopt(short, long, default_value = "http://127.0.0.1:1521")]
    pub db_server: String,

    /// the keystore for signing
    #[structopt(short, long, default_value = "//Alice")]
    pub key_store: String,
}

pub fn get_app<'a, 'b>() -> App<'a, 'b> {
    App::new("stake-wallet")
        .author("Parallel Team")
        .about("Multi signature wallet for staking.")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommands(vec![
            SubCommand::with_name("start").about("Start client"),
            SubCommand::with_name("getaddress").about("Print account address"),
            SubCommand::with_name("getmultiaddress")
                .about("Print multi signature account addresses"),
            SubCommand::with_name("show")
                .about("Print detail information of wallet")
                .arg(
                    Arg::with_name("name")
                        .help("The name of keystore file")
                        .required(true),
                )
                .arg(
                    Arg::with_name("password")
                        .help("The password you want to set")
                        .required(true),
                )
                .arg(
                    Arg::with_name("threshold")
                        .help("The threshold of multi signature account")
                        .required(true),
                )
                .arg(
                    Arg::with_name("address")
                        .help("The address of keystore")
                        .required(true),
                )
                .arg(
                    Arg::with_name("others")
                        .help("The other signer address of multi signature account")
                        .required(true),
                ),
            SubCommand::with_name("create")
                .about("Submit a transfer transaction")
                .args_from_usage(
                    "
                <name>  'The name of keystore file'
                <threshold>  'The threshold of multi signature account'
                <seed>  'The seed of keystore'
                <others>  'The other signer address of multi signature account'
          ",
                ),
        ])
}
