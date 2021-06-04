use super::multisig::Multisig;
use super::system::System;
pub use substrate_subxt::DefaultNodeRuntime as KusamaRuntime;

impl Multisig for KusamaRuntime {}
impl System for KusamaRuntime {}
