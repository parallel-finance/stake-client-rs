use async_std::task;

/// The first wallet to call withdraw. No need use 'TimePoint' and call 'approve_as_multi'.
pub(crate) async fn do_first_withdraw() {
    println!("do_first_withdraw");
}

/// If the wallet is the middle one to call withdraw, need to get 'TimePoint' and call 'approve_as_multi'.
pub(crate) async fn do_middle_withdraw() {
    println!("do_middle_withdraw");
}

/// If the wallet is the last one need to get 'TimePoint' and call 'as_multi'.
pub(crate) async fn do_last_withdraw() {
    println!("do_last_withdraw");
}
