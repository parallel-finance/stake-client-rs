use super::heiko_staking::Staking as HeikoStaking;
use super::multisig::Multisig;
use codec::{Decode, Encode};
pub use substrate_subxt::NodeTemplateRuntime as HeikoRuntime;

impl Multisig for HeikoRuntime {}

impl HeikoStaking for HeikoRuntime {}
