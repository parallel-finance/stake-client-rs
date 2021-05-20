pub use substrate_subxt::DefaultNodeRuntime as RelayRuntime;
use super::multisig::Multisig;

impl Multisig for RelayRuntime {}
