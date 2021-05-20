use thiserror::Error as ThisError;
use toml::de::Error as TomlError;

use core::fmt::Error as SerializeError;
use std::io::Error as IoError;
use db::error::Error as DbError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Io Error: `{0:?}`")]
    IoError(#[from] IoError),
    #[error("Toml Error: `{0:?}`")]
    TomlError(#[from] TomlError),
    #[error("Serialize Error: `{0:?}`")]
    SerializeError(#[from] SerializeError),
    #[error("DB Error: `{0:?}`")]
    DbError(#[from] DbError),
}

pub type Result<T> = std::result::Result<T, Error>;
