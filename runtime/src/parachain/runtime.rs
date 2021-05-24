use super::vanilla_oracle::VanillaOracle;
use codec::{Decode, Encode};
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

impl VanillaOracle for ParaRuntime {
    type OracleKey = CurrencyId;
    type OracleValue = Price;
}
