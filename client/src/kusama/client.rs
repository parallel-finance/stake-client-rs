use crate::common::error::Error;
use crate::common::primitives::AccountId;
use crate::keystore::{crypto::Sr25519, wallet::get_keystore};
use crate::kusama::{listener, tasks};

use async_std::sync::{Arc, Mutex};
use frame_support::PalletId;
use futures::join;
use log::{error, info};
use parallel_primitives::{Balance, CurrencyId, PriceWithDecimal};
use runtime::heiko::runtime::HeikoRuntime;
use runtime::kusama::runtime::KusamaRuntime;
use sp_core::sr25519::Pair;
use structopt::StructOpt;
use substrate_subxt::{staking::Staking, system::System, ClientBuilder, PairSigner};
use tokio::sync::{mpsc, oneshot};
use xcm::v0::{MultiLocation, Outcome};

pub const LISTEN_INTERVAL: u64 = 24000; // 6 * block_time
pub const TASK_INTERVAL: u64 = 6000;
pub const MIN_BOND_BALANCE: u128 = 100_000_000_000_000;

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

impl StartRelayCmd {
    pub async fn run(&self) {
        // get pair
        let password: Option<String>;
        match &self.password {
            Some(p) => password = Some(p.to_string()),
            None => password = rpassword::read_password_from_tty(Some("Type password:")).ok(),
        }

        // get keystore
        let keystore = get_keystore(self.key_store.to_string()).unwrap();
        info!("{:?}", keystore);

        let pair = keystore.into_pair::<Sr25519>(password).unwrap();

        // get other signatories
        let other_signatories = keystore.get_other_signatories().unwrap();

        let temporary_cmd = TemporaryCmd {
            relay_ws_server: self.relay_ws_server.clone(),
            para_ws_server: self.para_ws_server.clone(),
            relay_key_pair: pair.clone(),
            para_key_pair: pair.clone(),
            relay_pool_addr: self.relay_pool_addr.clone(),
            para_pool_addr: self.para_pool_addr.to_string(),
            relay_multi_other_signatories: other_signatories.clone(),
            para_multi_other_signatories: other_signatories.clone(),
            first: self.first,
        };
        let r = run(&temporary_cmd).await;
        info!("relaychain client finished {:?}", r);
    }
}

pub enum TasksType {
    RelayBond,
    RelayBondExtra,
    ParaRecordRewards(Amount),
    ParaRecordSlash(Amount),
    ParaUnstake(AccountId, Amount),
    RelayUnbonded(AccountId, Amount),
    RelayEraIndexChanged(u32),
    RelayWithdrawUnbonded(AccountId, Amount),
}
pub type Amount = u128;

//todo this is a TemporaryCmd receive arguments
pub struct TemporaryCmd {
    pub relay_ws_server: String,
    pub para_ws_server: String,
    pub relay_key_pair: Pair,
    pub para_key_pair: Pair,
    pub relay_pool_addr: String,
    pub para_pool_addr: String,
    pub relay_multi_other_signatories: Vec<AccountId>,
    pub para_multi_other_signatories: Vec<AccountId>,
    pub first: bool,
}

pub async fn run(cmd: &TemporaryCmd) -> Result<(), Error> {
    // initial relaychain client
    let relay_subxt_client = ClientBuilder::<KusamaRuntime>::new()
        .set_url(cmd.relay_ws_server.clone())
        .register_type_size::<<KusamaRuntime as System>::AccountId>("T::AccountId")
        .register_type_size::<<KusamaRuntime as Staking>::CandidateReceipt>(
            "CandidateReceipt<Hash>",
        )
        .register_type_size::<u32>("CoreIndex")
        .register_type_size::<u32>("GroupIndex")
        .register_type_size::<PalletId>("ParaId")
        .register_type_size::<MultiLocation>("MultiLocation")
        .register_type_size::<Outcome>("xcm::v0::Outcome")
        .register_type_size::<Outcome>("Outcome")
        .register_type_size::<([u8; 4], u64)>("MessageId")
        .skip_type_sizes_check()
        .build()
        .await
        .map_err(|e| {
            error!("subxt_client error: {:?}", e);
            Error::SubxtError(e)
        })?;
    // let pair = Pair::from_string(cmd.relay_key_store, None).unwrap();
    let pair = cmd.relay_key_pair.clone();
    let relay_signer = PairSigner::<KusamaRuntime, Pair>::new(pair);
    // initial parachain client
    let para_subxt_client = ClientBuilder::<HeikoRuntime>::new()
        .set_url(cmd.para_ws_server.clone())
        .register_type_size::<CurrencyId>("CurrencyIdOf<T>")
        .register_type_size::<Balance>("BalanceOf<T>")
        .register_type_size::<<HeikoRuntime as System>::AccountId>("T::AccountId")
        .register_type_size::<CurrencyId>("T::CurrencyId")
        .register_type_size::<Balance>("T::Balance")
        .register_type_size::<CurrencyId>("T::OracleKey")
        .register_type_size::<PriceWithDecimal>("T::OracleValue")
        .register_type_size::<CurrencyId>("CurrencyId")
        .register_type_size::<PalletId>("ParaId")
        .register_type_size::<MultiLocation>("MultiLocation")
        .register_type_size::<Outcome>("xcm::v0::Outcome")
        .register_type_size::<Outcome>("Outcome")
        .register_type_size::<([u8; 4], u64)>("MessageId")
        .skip_type_sizes_check()
        .build()
        .await
        .map_err(|e| {
            error!("subxt_client error: {:?}", e);
            Error::SubxtError(e)
        })?;
    // let pair = Pair::from_string(cmd.para_key_store, None).unwrap();
    let pair = cmd.para_key_pair.clone();
    let para_signer = PairSigner::<HeikoRuntime, Pair>::new(pair);
    // initial channel
    let (system_rpc_tx, system_rpc_rx) = mpsc::channel::<(TasksType, oneshot::Sender<u64>)>(10);

    // todo put this to database, because this will be lost when the client restart
    let withdraw_unbonded_amount = Arc::new(Mutex::new(0));

    // initial multi threads to listen on-chain status
    let l = listener::listener(
        &relay_subxt_client,
        &para_subxt_client,
        system_rpc_tx,
        cmd.relay_pool_addr.clone(),
        withdraw_unbonded_amount.clone(),
    );

    // initial task to receive order and dive
    let t = tasks::dispatch(
        &relay_subxt_client,
        &para_subxt_client,
        &relay_signer,
        &para_signer,
        system_rpc_rx,
        cmd.relay_multi_other_signatories.clone(),
        cmd.relay_pool_addr.clone(),
        cmd.para_pool_addr.clone(),
        cmd.first,
        withdraw_unbonded_amount.clone(),
    );
    join!(l, t);
    Ok(())
}
