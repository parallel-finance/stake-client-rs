mod listener;
mod tasks;
mod transaction;

use crate::error::Error;
use crate::primitives::AccountId;

use frame_support::PalletId;
use futures::join;
use log::error;
use parallel_primitives::{Balance, CurrencyId, PriceWithDecimal};
use runtime::heiko::{self, runtime::HeikoRuntime};
use runtime::kusama::{self, runtime::KusamaRuntime};
use runtime::pallets::multisig::Multisig;
use sp_core::sr25519::Pair;
use substrate_subxt::{staking::Staking, system::System, ClientBuilder, PairSigner};
use tokio::sync::{mpsc, oneshot};
use xcm::v0::{MultiLocation, Outcome};

pub const LISTEN_INTERVAL: u64 = 24000; // 6 * block_time
pub const TASK_INTERVAL: u64 = 6000;
pub const MIN_BOND_BALANCE: u128 = 100_000_000_000_000;

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
    let mut withdraw_unbonded_amount: Amount = 0;

    // initial multi threads to listen on-chain status
    let l = listener::listener(
        &relay_subxt_client,
        &para_subxt_client,
        system_rpc_tx,
        cmd.relay_pool_addr.clone(),
        withdraw_unbonded_amount,
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
        &mut withdraw_unbonded_amount,
    );
    join!(l, t);
    Ok(())
}
