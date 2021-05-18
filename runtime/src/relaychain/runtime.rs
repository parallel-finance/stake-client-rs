pub use substrate_subxt::NodeTemplateRuntime as RelayRuntime;
use super::test_oracle::TestOracle;


impl TestOracle for RelayRuntime {
    type OracleKey = u128;
    type OracleValue = u128;
    type Test = u64;
}
