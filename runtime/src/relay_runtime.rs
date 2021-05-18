use codec::Encode;
use frame_support::Parameter;
use sp_runtime::traits::Member;
use substrate_subxt::system::System;
pub use substrate_subxt::NodeTemplateRuntime as RelayRuntime;

#[module]
pub trait TestOracle: System {
    /// The data key type
    type OracleKey: Parameter + Member;

    /// The data value type
    type OracleValue: Parameter + Member + Ord;

    type Test: Parameter + Member + Ord;
}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct Feed1Values<T: TestOracle> {
    pub values: Vec<(T::OracleKey, T::OracleValue)>,
}

impl TestOracle for RelayRuntime {
    type OracleKey = u128;
    type OracleValue = u128;
    type Test = u64;
}
