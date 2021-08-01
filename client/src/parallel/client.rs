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
        .register_type_size::<[u8; 4]>("ParaId")
        .register_type_size::<MultiLocation>("MultiLocation")
        .register_type_size::<Outcome>("xcm::v0::Outcome")
        .register_type_size::<Outcome>("Outcome")
        .register_type_size::<[u8; 32]>("MessageId")
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
#[tokio::test]
async fn test_decode_event() -> Result<(), Error> {
    let relay_ws_server = "ws://localhost:9944";

    let storage_change = hex::decode("1400000000000000b814fb0a0000000002000000010000003204d007000001000000470000000000010000002c01d0070000221164d61bfc7eb4705b310ecef08194c63a06c7102d4c08bb4e1aac6e2798f6746c1ea40ff59943a337fb7204bcfd0b9acc5d150ff78ce06c670a159191ba3238b5eec10bb4d7645a3b8286bb6eddc04e0a2ffcf32c6e061528acfe775914424eb1d74c4245bcf10db6759e4a1fd646ec868bfc41b1f227e14e9191cefcd51f00ad4c4ce5ce3b524790412b63b3fd3b4d2355227257ad2c962b40ae18468df18c9f4977a24ec47c848bc12e7c404694e969d898fd49c257be74dea04fb45640232def20f8242faacceded6e1e012f5552f7fd1cdfab71864477bde31551648ce12d8c2fa64bea26b327282d0833fb4028be61a99b005d8461504bdd18f260b74b1010ccb364518bff212956c32599860b91016c3044a9a27b9d4ea9cc3a1c620a110cdabf7f6cd59e8010c9cae4b24c3b26f7775a8679e1ff454d8cdb9209f8dd0268f6b8ec5ab3aee90494b740ad82e07c836c2bd7f9ce626908e7fdc56d1f0e6020db70e42ed6f4f6e21d253e2f509c6825a81c918d024a2f8f3fea585ac7eb5bc6d2b83418099a7a8da309a6d1df7f6716aeee1cfd26234115e4bbef09d5e0bf68080661757261202bcd1508000000000561757261010106db3098bcc504e5e5172465c0d99ee1a5cb95bc66be5b4737139d6b8ebf5d6aeb92cfe9be8d8aab7aae3247d0e44ef6367164ca69edc367b2cf711302fef48300000000000000000000010000003202c17c0f4e580dbc0ae5b37fbb1d42486cc4b65cfe00cd5870384ecbaed6dfb1bd0180c3c9010000000010000001000000000080b2e60e00000000020000").unwrap();

    let relay_subxt_client = ClientBuilder::<RelayRuntime>::new()
        .set_url(relay_ws_server)
        .register_type_size::<<RelayRuntime as System>::AccountId>("T::AccountId")
        .register_type_size::<<RelayRuntime as Staking>::CandidateReceipt>("CandidateReceipt<Hash>")
        .register_type_size::<u32>("CoreIndex")
        .register_type_size::<u32>("GroupIndex")
        .register_type_size::<[u8; 4]>("ParaId")
        .register_type_size::<MultiLocation>("MultiLocation")
        .register_type_size::<Outcome>("xcm::v0::Outcome")
        .register_type_size::<Outcome>("Outcome")
        .register_type_size::<[u8; 32]>("MessageId")
        .skip_type_sizes_check()
        // .register_type_size::<([u8; 20])>("EthereumAddress")
        .build()
        .await
        .unwrap();

    let decoder = relay_subxt_client.events_decoder();
    // decoder.decode_events(&mut storage_change.clone().as_bytes())?;
    decoder.decode_events(&mut storage_change.clone().as_slice())?;
    Ok(())
}
