use crate::crypto::*;
use crate::pkcs8;
use crate::primitives::{AccountId, MIN_POOL_BALANCE};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Keystore {
    // the address of keystore.
    pub address: String,

    // the multi-address of current and others
    pub multi_address: String,

    // the other address of multi-signature accounts.
    pub others: Vec<String>,

    // the threshold of multi-signature.
    pub threshold: u16,

    // The network of keystore, for 'polkadot' or 'ksm'.
    pub network: String,

    // the encoded data of keystore.
    pub encoded: String,
}

impl Keystore {
    pub fn parse_from_file(path: String) -> Result<Self, ()> {
        let data = fs::read_to_string(path).map_err(|_| ())?;
        let keystore: Self = serde_json::from_str(&data).map_err(|_| ())?;
        Ok(keystore)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn encoded_bytes(&self) -> Vec<u8> {
        let encoded = if self.encoded.starts_with("0x") {
            &self.encoded[2..]
        } else {
            &self.encoded
        };
        hex::decode(encoded).unwrap_or(vec![])
    }

    pub fn into_pair<T: Crypto>(&self, password: Option<String>) -> Result<T::Pair, ()> {
        let encoded = self.encoded_bytes();
        if encoded.is_empty() {
            return Err(());
        }
        match pkcs8::decode(&encoded[..], password) {
            Ok((_, secret_key)) => T::pair_from_seed_slice(&secret_key[..]),
            Err(_) => Err(()),
        }
    }

    pub fn get_other_signatories(&self) -> Result<Vec<AccountId>, ()> {
        let mut other_signatories: Vec<AccountId> = vec![];
        for a in self.others.iter() {
            let account_id = AccountId::from_string(&a).map_err(|_err| ())?;
            other_signatories.push(account_id);
        }
        other_signatories.sort_by(|a, b| a.cmp(&b));
        Ok(other_signatories)
    }
}
