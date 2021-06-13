mod listener;
mod tasks;
pub mod transaction;
use crate::error::Error;
use crate::primitives::AccountId;
use futures::join;
use log::error;
use runtime::heiko::{self, runtime::HeikoRuntime};
use runtime::kusama::{self, runtime::KusamaRuntime};
use runtime::pallets::multisig::Multisig;
use sp_core::sr25519::Pair;
use sp_utils::mpsc::tracing_unbounded;
use substrate_subxt::{system::System, ClientBuilder, PairSigner};
pub const LISTEN_INTERVAL: u64 = 24000; // 6 * block_time
pub const TASK_INTERVAL: u64 = 6000;
pub const MIN_BOND_BALANCE: u128 = 100_000_000_000_000;
pub enum TasksType {
    RelayBond,
    RelayBondExtra,
    ParaRecordRewards(Amount),
    ParaRecordSlash(Amount),
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
    let subxt_relay_client = ClientBuilder::<KusamaRuntime>::new()
        .set_url(cmd.relay_ws_server.clone())
        .register_type_size::<<KusamaRuntime as System>::AccountId>("T::AccountId")
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
    let subxt_para_client = ClientBuilder::<HeikoRuntime>::new()
        .set_url(cmd.para_ws_server.clone())
        .register_type_size::<<KusamaRuntime as System>::AccountId>("T::AccountId")
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
    let (system_rpc_tx, system_rpc_rx) = tracing_unbounded::<TasksType>("mpsc_system_rpc");

    // initial multi threads to listen on-chain status
    let l = listener::listener(
        &subxt_relay_client,
        system_rpc_tx,
        cmd.relay_pool_addr.clone(),
    );

    // initial task to receive order and dive
    let t = tasks::dispatch(
        &subxt_relay_client,
        &subxt_para_client,
        &relay_signer,
        &para_signer,
        system_rpc_rx,
        cmd.relay_multi_other_signatories.clone(),
        cmd.relay_pool_addr.clone(),
        cmd.first,
    );
    join!(l, t);
    Ok(())
}
