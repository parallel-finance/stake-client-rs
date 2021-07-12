use super::currencies::Currencies;
use super::multisig::Multisig;
use super::staking::Staking;
use super::system::System;
use crate::pallets::xcm_pallet::XcmPallet;
pub use substrate_subxt::DefaultNodeRuntime as KusamaRuntime;

impl Multisig for KusamaRuntime {}
impl System for KusamaRuntime {}
impl Staking for KusamaRuntime {}
impl Currencies for KusamaRuntime {}
impl XcmPallet for KusamaRuntime {}
