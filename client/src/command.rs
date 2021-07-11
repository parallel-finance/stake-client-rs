use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "stake_client", about = "Utility for stake client")]
pub enum StakeClient {
    /// Create keystore file
    Create(CreateCmd),

    /// Run para chain multi-sig account
    StartPara(StartParaCmd),

    /// Run relay chain multi-sig account
    StartRelay(StartRelayCmd),
}

#[derive(Debug, StructOpt)]
pub struct CreateCmd {
    /// the keystore name
    #[structopt(short, long, default_value = "keystore")]
    pub name: String,

    /// the threshold of multi-signature accounts
    #[structopt(short, long)]
    pub threshold: u16,

    /// the other signatories of multi-signature accounts
    #[structopt(short, long)]
    pub other_signatories: Vec<String>,
}

#[derive(Debug, StructOpt)]
pub struct StartParaCmd {
    /// the keystore for signing
    #[structopt(short, long, default_value = "keystore.json")]
    pub key_store: String,

    /// websocket server endpoint of para chain
    #[structopt(long, default_value = "ws://127.0.0.1:9944")]
    pub para_ws_server: String,

    /// websocket server endpoint of relay chain
    #[structopt(long, default_value = "ws://127.0.0.1:9955")]
    pub relay_ws_server: String,

    /// data base server endpoint
    #[structopt(short, long, default_value = "http://127.0.0.1:1521")]
    pub db_server: String,

    /// pool address of para chain
    #[structopt(
        long,
        default_value = "5EYCAe5iie3Jn5XKaz6Q2bumoE3whfem8PUFtkVzeSq1yLoH"
    )]
    pub para_pool_addr: String,

    /// the password of keystore
    #[structopt(short, long)]
    pub password: Option<String>,

    /// temp use to decide which account create first multi-signature transaction
    #[structopt(short, long)]
    pub first: bool,
}

#[derive(Debug, StructOpt)]
pub struct StartRelayCmd {
    /// the keystore for signing
    #[structopt(short, long, default_value = "keystore.json")]
    pub key_store: String,

    /// websocket server endpoint of para chain
    #[structopt(long, default_value = "ws://127.0.0.1:9944")]
    pub para_ws_server: String,

    /// websocket server endpoint of relay chain
    #[structopt(long, default_value = "ws://127.0.0.1:9955")]
    pub relay_ws_server: String,

    /// data base server endpoint
    #[structopt(short, long, default_value = "http://127.0.0.1:1521")]
    pub db_server: String,

    /// pool address of relay chain
    #[structopt(
        long,
        default_value = "5DjYJStmdZ2rcqXbXGX7TW85JsrW6uG4y9MUcLq2BoPMpRA7"
    )]
    pub relay_pool_addr: String,

    /// pool address of para chain
    #[structopt(
        long,
        default_value = "5EYCAe5iie3Jn5XKaz6Q2bumoE3whfem8PUFtkVzeSq1yLoH"
    )]
    pub para_pool_addr: String,

    /// the password of keystore
    #[structopt(short, long)]
    pub password: Option<String>,

    /// temp use to decide which account create first multi-signature transaction
    #[structopt(short, long)]
    pub first: bool,
}
