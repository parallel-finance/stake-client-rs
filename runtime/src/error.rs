use substrate_subxt::Error as SubxtError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Substrate Subxt Error: `{0:?}`")]
    SubxtError(#[from] SubxtError),
}
