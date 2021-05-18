//parachain_multisig pallet
//parachain_stake

//TODO 定义本地trait， 引入外部struct， 在本地impl新的runtime
use codec::{Decode, Encode};
use frame_support::Parameter;
use sp_runtime::traits::Member;
use substrate_subxt::system::System;
pub use substrate_subxt::NodeTemplateRuntime as ParaRuntime;

use sp_runtime::FixedU128;
pub type Price = FixedU128;
#[derive(Encode, Decode, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum CurrencyId {
    DOT,
    KSM,
    USDT,
    #[allow(non_camel_case_types)]
    xDOT,
    Native,
}

#[module]
pub trait VanillaOracle: System {
    /// The data key type
    type OracleKey: Parameter + Member;

    /// The data value type
    type OracleValue: Parameter + Member + Ord;
}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct FeedValues<T: VanillaOracle> {
    pub values: Vec<(T::OracleKey, T::OracleValue)>,
}

impl VanillaOracle for ParaRuntime {
    type OracleKey = CurrencyId;
    type OracleValue = Price;
}