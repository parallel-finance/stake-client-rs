table! {
    withdraw (idx) {
        idx -> Int4,
        state -> Varchar,
        created_at -> Timestamp,
    }
}

table! {
    withdraw_tx (idx) {
        idx -> Int4,
        tx_hash -> Varchar,
        pool -> Varchar,
        multisig_origin -> Varchar,
        amount -> Varchar,
    }
}

joinable!(withdraw_tx -> withdraw (idx));

allow_tables_to_appear_in_same_query!(
    withdraw,
    withdraw_tx,
);
