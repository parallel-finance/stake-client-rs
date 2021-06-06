use crate::crypto::*;
use crate::keystore::Keystore;
use crate::pkcs8;
use crate::primitives::AccountId;
use sp_core::{blake2_256, crypto::Ss58Codec, hexdisplay::HexDisplay, Decode, Encode};

pub fn get_keystore(path: String) -> Result<Keystore, Box<dyn std::error::Error>> {
    let k = Keystore::parse_from_file(path).map_err(|_err| "failed to get keystore from file")?;
    Ok(k)
}

pub fn create_keystore(
    password: Option<String>,
    threshold: u16,
    seed: String,
    others: Vec<String>,
) -> Result<Keystore, Box<dyn std::error::Error>> {
    // encoded data
    // let seed_hex = &hex::decode(seed).map_err(|_err| "invalid seed")?;
    // let pair = Sr25519::pair_from_seed(&seed_hex)
    let pair = Sr25519::pair_from_seed(&seed).map_err(|_err| "failed to create pair from seed")?;
    let (public_key, secret_key) = (pair.public().to_raw_vec(), pair.to_raw_vec());
    let encoded = pkcs8::encode(&secret_key[..], &public_key[..], password)
        .map_err(|_err| "failed to encode pair")?;
    // let addr = pair.public().to_ss58check();
    println!(
        "public key:{}",
        format!("0x{}", HexDisplay::from(&public_key.as_ref()))
    );

    // multi signature address
    let mut other_signatories: Vec<AccountId> = vec![];
    for a in others.iter() {
        let account_id = AccountId::from_string(&a).map_err(|_err| "invalid other account")?;
        other_signatories.push(account_id);
    }
    let signatories = ensure_sorted_and_insert(other_signatories, pair.public().into())
        .map_err(|_err| "failed to sort signatories")?;
    let id = multi_account_id(&signatories, threshold.clone());

    let k = Keystore {
        address: format!("{}", pair.public()),
        multi_address: id.to_string(),
        others,
        threshold,
        network: "ksm".to_string(),
        encoded: format!("0x{}", hex::encode(encoded)),
    };
    Ok(k)
}

fn ensure_sorted_and_insert(
    other_signatories: Vec<AccountId>,
    who: AccountId,
) -> Result<Vec<AccountId>, String> {
    let mut signatories = other_signatories;
    let mut maybe_last = None;
    let mut index = 0;
    for item in signatories.iter() {
        if let Some(last) = maybe_last {
            if last >= item {
                return Err("SignatoriesOutOfOrder".into());
            }
        }
        if item <= &who {
            if item == &who {
                return Err("SenderInSignatories".into());
            }
            index += 1;
        }
        maybe_last = Some(item);
    }
    signatories.insert(index, who);
    Ok(signatories)
}

fn multi_account_id(who: &[AccountId], threshold: u16) -> AccountId {
    let entropy = (b"modlpy/utilisuba", who, threshold).using_encoded(blake2_256);
    AccountId::decode(&mut &entropy[..]).unwrap_or_default()
}
