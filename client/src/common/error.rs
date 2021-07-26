use thiserror::Error as ThisError;
use toml::de::Error as TomlError;

use core::fmt::Error as SerializeError;
use runtime::error::Error as ClientRuntimeError;
use std::io::Error as IoError;
use substrate_subxt::Error as SubxtError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Io Error: `{0:?}`")]
    IoError(#[from] IoError),
    #[error("Toml Error: `{0:?}`")]
    TomlError(#[from] TomlError),
    #[error("Serialize Error: `{0:?}`")]
    SerializeError(#[from] SerializeError),
    #[error("Substrate Subxt Error: `{0:?}`")]
    SubxtError(#[from] SubxtError),
    #[error("Client runtime Error: `{0:?}`")]
    ClientRuntimeError(#[from] ClientRuntimeError),
    #[error("Other error: {0}")]
    Other(String),
}
