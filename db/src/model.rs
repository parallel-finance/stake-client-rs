use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::schema::{withdraw, withdraw_tx};

#[derive(
    AsChangeset, Eq, PartialEq, Debug, Serialize, Deserialize, Queryable, Identifiable, Insertable,
)]
#[table_name = "withdraw"]
#[primary_key(idx)]
pub struct Withdraw {
    pub idx: i32,
    pub state: String,
    pub created_at: NaiveDateTime,
    pub height: Option<i32>,
    pub sig_count: Option<i32>,
}

#[derive(
    AsChangeset, Eq, PartialEq, Debug, Serialize, Deserialize, Queryable, Identifiable, Insertable,
)]
#[table_name = "withdraw_tx"]
#[primary_key(idx)]
pub struct WithdrawTx {
    pub idx: i32,
    pub tx_hash: String,
    pub pool: String,
    pub multisig_origin: String,
    pub amount: String,
}
