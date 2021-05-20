use super::multisig::Multisig;
pub use substrate_subxt::DefaultNodeRuntime as RelayRuntime;

impl Multisig for RelayRuntime {}
