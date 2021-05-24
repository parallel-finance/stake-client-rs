use super::vanilla_oracle::VanillaOracle;
use codec::{Decode, Encode};
pub use substrate_subxt::NodeTemplateRuntime as HeikoRuntime;

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

impl VanillaOracle for HeikoRuntime {
    type OracleKey = CurrencyId;
    type OracleValue = Price;
}
