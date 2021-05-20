use thiserror::Error as ThisError;
use substrate_subxt::Error as SubxtError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Substrate Subxt Error: `{0:?}`")]
    SubxtError(#[from] SubxtError),
}