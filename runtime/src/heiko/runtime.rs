use super::currencies::Currencies;
use super::liquid_staking::LiquidStaking;
use super::multisig::Multisig;
use super::orml_tokens::Tokens;
use super::system::System;
pub use parallel_primitives::CurrencyId;
pub use substrate_subxt::NodeTemplateRuntime as HeikoRuntime;

impl Multisig for HeikoRuntime {}

impl LiquidStaking for HeikoRuntime {}

impl Tokens for HeikoRuntime {
    type CurrencyId = CurrencyId;
}

impl System for HeikoRuntime {}

impl Currencies for HeikoRuntime {}
