use codec::Encode;
use parallel_primitives::CurrencyId;
use substrate_subxt::balances::Balances;
use substrate_subxt::system::System;

#[module]
pub trait Currencies: Balances {}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct TransferCall<'a, T: Currencies> {
    /// Destination of the transfer.
    pub dest: &'a <T as System>::Address,
    pub currency_id: CurrencyId,
    /// Amount to transfer.
    #[codec(compact)]
    pub amount: T::Balance,
}

pub fn currencies_transfer_call<'a, T: Currencies + System>(
    dest: &'a <T as System>::Address,
    currency_id: CurrencyId,
    amount: T::Balance,
) -> TransferCall<T> {
    TransferCall::<T> {
        dest,
        currency_id,
        amount,
    }
}
