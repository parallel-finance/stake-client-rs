use codec::Encode;
use frame_system::AccountInfo;
use substrate_subxt::balances::Balances;
use substrate_subxt::system::System as SubxtSystem;

#[derive(Encode, Copy, Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Store)]
pub struct AccountStore<T: System> {
    #[store(returns =AccountInfo<T::Index, T::AccountData>)]
    pub account: T::AccountId,
}

#[module]
pub trait System: Balances + SubxtSystem {}
