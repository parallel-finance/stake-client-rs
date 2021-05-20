use diesel::result::{ConnectionError, Error as DieselError};
use r2d2::Error as R2D2Error;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Postgres Connection Error: `{0:?}`")]
    PgConnError(#[from] ConnectionError),
    #[error("Postgres Connection Pool Error: `{0:?}`")]
    PgConnPoolError(#[from] R2D2Error),
    #[error("Postgres Runtime Error: `{0:?}`")]
    PgRuntimeError(#[from] DieselError),
    #[error("Postgres Connection Pool Not Ready Error")]
    PgPoolNotReady,
}

pub type Result<T> = std::result::Result<T, Error>;
