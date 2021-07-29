use super::currencies::Currencies;
use super::liquid_staking::LiquidStaking;
use super::multisig::Multisig;
use super::nominee_election::NomineeElection;
use super::orml_tokens::Tokens;
use super::system::System;
use crate::pallets::xcm_pallet::XcmPallet;
pub use parallel_primitives::CurrencyId;
pub use substrate_subxt::NodeTemplateRuntime as HeikoRuntime;

impl Multisig for HeikoRuntime {}

impl LiquidStaking for HeikoRuntime {}

impl NomineeElection for HeikoRuntime {
    type MaxValidators = u32;
}

impl Tokens for HeikoRuntime {
    type CurrencyId = CurrencyId;
}

impl System for HeikoRuntime {}

impl Currencies for HeikoRuntime {}

impl XcmPallet for HeikoRuntime {}
