use crate::keystore::wallet::CreateCmd;
use crate::kusama::client::StartRelayCmd;
use crate::parallel::client::StartParaCmd;
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
