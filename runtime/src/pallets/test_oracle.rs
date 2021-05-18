use codec::Encode;
use frame_support::Parameter;
use sp_runtime::traits::Member;
use substrate_subxt::system::System;
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