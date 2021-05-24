use substrate_subxt::balances::{Balances, TransferCall};
use substrate_subxt::system::System;

pub fn balances_transfer_call<'a, T: Balances + System>(
    to: &'a <T as System>::Address,
    amount: T::Balance,
) -> TransferCall<T> {
    TransferCall::<T> { to, amount }
}
