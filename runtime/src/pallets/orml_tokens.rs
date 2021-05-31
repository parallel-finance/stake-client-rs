use codec::Encode;
use frame_support::Parameter;
use orml_tokens::AccountData;
use sp_runtime::traits::Member;
use substrate_subxt::balances::Balances;

#[derive(Encode, Copy, Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Store)]
pub struct AccountsStore<T: Tokens> {
    #[store(returns =AccountData<T::Balance>)]
    pub account: T::AccountId,
    pub currency_id: T::CurrencyId,
}

#[module]
pub trait Tokens: Balances {
    type CurrencyId: Parameter + Member;
}
