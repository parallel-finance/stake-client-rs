use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Cmd {
    /// websocket server endpoint
    #[structopt(short, long, default_value = "ws://127.0.0.1:9944")]
    pub ws_server: String,

    /// the keystore for signing
    #[structopt(short, long, default_value="//Alice")]
    pub key_store: String,
}