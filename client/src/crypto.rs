pub use sp_core::{
    crypto::{set_default_ss58_version, AccountId32, Derive, Ss58AddressFormat, Ss58Codec},
    ecdsa, ed25519, sr25519, Pair, Public,
};

pub trait Crypto: Sized {
    type Pair: Pair<Public = Self::Public>;
    type Public: Public + Ss58Codec + AsRef<[u8]> + std::hash::Hash;

    fn pair_from_seed(seed: &str) -> Result<Self::Pair, ()>;

    fn pair_from_seed_slice(slice: &[u8]) -> Result<Self::Pair, ()>;

    fn address<P: Pair>(pair: &P) -> String;
}

pub struct Sr25519;
impl Crypto for Sr25519 {
    type Pair = sr25519::Pair;
    type Public = sr25519::Public;

    fn pair_from_seed(seed: &str) -> Result<Self::Pair, ()> {
        match Self::Pair::from_string(seed, None) {
            Ok(pair) => Ok(pair),
            Err(_) => Err(()),
        }
    }

    fn pair_from_seed_slice(slice: &[u8]) -> Result<Self::Pair, ()> {
        match Self::Pair::from_seed_slice(slice).map_err(|_| ()) {
            Ok(pair) => Ok(pair),
            Err(_) => {
                let sec = schnorrkel::SecretKey::from_ed25519_bytes(slice).map_err(|_| ())?;
                Ok(Self::Pair::from(sec))
            }
        }
    }

    fn address<P: Pair>(pair: &P) -> String {
        pair.public().to_ss58check()
    }
}
