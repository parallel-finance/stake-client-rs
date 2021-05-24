use super::multisig::Multisig;
pub use substrate_subxt::DefaultNodeRuntime as KusamaRuntime;

impl Multisig for KusamaRuntime {}
