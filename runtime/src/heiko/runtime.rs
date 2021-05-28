use super::liquid_staking::LiquidStaking;
use super::multisig::Multisig;
use codec::{Decode, Encode};
pub use substrate_subxt::NodeTemplateRuntime as HeikoRuntime;

impl Multisig for HeikoRuntime {}

impl LiquidStaking for HeikoRuntime {}
