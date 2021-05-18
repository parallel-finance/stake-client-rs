use codec::Encode;
use frame_support::Parameter;
use sp_runtime::traits::Member;
use substrate_subxt::system::System;
#[module]
pub trait Multisig: System {
    /// The data key type
    type OracleKey: Parameter + Member;

    /// The data value type
    type OracleValue: Parameter + Member + Ord;
}

// #[derive(Clone, Debug, PartialEq, Call, Encode)]
// pub struct FeedValues<T: VanillaOracle> {
//     pub values: Vec<(T::OracleKey, T::OracleValue)>,
// }