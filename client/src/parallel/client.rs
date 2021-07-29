use crate::common::primitives::{AccountId, TasksType};
use crate::keystore::{crypto::Sr25519, wallet::get_keystore};
use crate::parallel::{listener, tasks};

use async_std::sync::{Arc, Mutex};
use frame_support::PalletId;
use futures::join;
use parallel_primitives::{Balance, CurrencyId, PriceWithDecimal};
use runtime::error::Error;
use runtime::heiko::{api::ValidatorSet, runtime::HeikoRuntime};
use runtime::kusama::runtime::KusamaRuntime as RelayRuntime;
use sp_core::crypto::Ss58Codec;
use structopt::StructOpt;
use substrate_subxt::staking::Staking;
use substrate_subxt::system::System;
use substrate_subxt::{ClientBuilder, PairSigner};
use tokio::sync::{mpsc, oneshot};
use xcm::v0::{MultiLocation, Outcome};

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
        default_value = "5EYCAe5iie3Jms55YSqwGAx8H5Yj4Xv84tWYmdbm1sB1EwtZ"
    )]
    pub para_pool_addr: String,

    /// the password of keystore
    #[structopt(short, long)]
    pub password: Option<String>,

    /// temp use to decide which account create first multi-signature transaction
    #[structopt(short, long)]
    pub first: bool,
}

impl StartParaCmd {
    pub async fn run(&self) {
        // get pair
        let password: Option<String>;
        match &self.password {
            Some(p) => password = Some(p.to_string()),
            None => password = rpassword::read_password_from_tty(Some("Type password:")).ok(),
        }

        // get keystore
        let keystore = get_keystore(self.key_store.to_string()).unwrap();
        println!("{:?}", keystore);

        let pair = keystore.into_pair::<Sr25519>(password).unwrap();

        // get other signatories
        let other_signatories = keystore.get_other_signatories().unwrap();
        let r = run(
            keystore.threshold,
            pair,
            other_signatories,
            &self.para_ws_server,
            &self.relay_ws_server,
            &self.para_pool_addr,
            &keystore.multi_address,
            CurrencyId::KSM,
            self.first,
        )
        .await;
        println!("para chain client finished:{:?}", r);
    }
}

pub async fn run(
    threshold: u16,
    pair: sp_core::sr25519::Pair,
    others: Vec<AccountId>,
    para_ws_server: &str,
    relay_ws_server: &str,
    pool_addr: &str,
    multi_addr: &str,
    currency_id: CurrencyId,
    first: bool,
) -> Result<(), Error> {
    // initialize heiko related api
    // todo register all unknown type
    let para_subxt_client = ClientBuilder::<HeikoRuntime>::new()
        .set_url(para_ws_server)
        .register_type_size::<CurrencyId>("CurrencyIdOf<T>")
        .register_type_size::<CurrencyId>("Currency<T>")
        .register_type_size::<CurrencyId>("Currency")
        .register_type_size::<Balance>("BalanceOf<T>")
        .register_type_size::<<HeikoRuntime as System>::AccountId>("T::AccountId")
        .register_type_size::<ValidatorSet<HeikoRuntime>>("ValidatorSet<T>")
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
        .unwrap();

    // todo register all unknown type
    let relay_subxt_client = ClientBuilder::<RelayRuntime>::new()
        .set_url(relay_ws_server)
        .register_type_size::<<RelayRuntime as System>::AccountId>("T::AccountId")
        .register_type_size::<<RelayRuntime as Staking>::CandidateReceipt>("CandidateReceipt<Hash>")
        .register_type_size::<u32>("CoreIndex")
        .register_type_size::<u32>("GroupIndex")
        .register_type_size::<PalletId>("ParaId")
        .register_type_size::<MultiLocation>("MultiLocation")
        .register_type_size::<Outcome>("xcm::v0::Outcome")
        .register_type_size::<Outcome>("Outcome")
        .register_type_size::<([u8; 4], u64)>("MessageId")
        .skip_type_sizes_check()
        // .register_type_size::<([u8; 20])>("EthereumAddress")
        .build()
        .await
        .unwrap();

    let multi_account_id = AccountId::from_string(multi_addr).unwrap();
    let pool_account_id = AccountId::from_string(pool_addr).unwrap();
    let para_signer = PairSigner::<HeikoRuntime, sp_core::sr25519::Pair>::new(pair.clone());

    // initial channel
    let (system_rpc_tx, system_rpc_rx) = mpsc::channel::<(TasksType, oneshot::Sender<u64>)>(10);

    // todo put this to database, because this will be lost when the client restart
    let withdraw_unbonded_amount = Arc::new(Mutex::new(0));

    // initial multi threads to listen on-chain status
    let l = listener::listener(
        system_rpc_tx,
        &para_subxt_client,
        &relay_subxt_client,
        pool_account_id.clone(),
        currency_id.clone(),
        withdraw_unbonded_amount.clone(),
    );

    // initial task to receive order and dive
    let t = tasks::dispatch(
        system_rpc_rx,
        &para_subxt_client,
        &relay_subxt_client,
        &para_signer,
        multi_account_id,
        pool_account_id,
        threshold,
        others,
        first,
        withdraw_unbonded_amount.clone(),
    );
    join!(l, t);
    Ok(())
}
