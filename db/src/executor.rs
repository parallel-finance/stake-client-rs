use crate::error::{Error, Result};

use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, HandleError, Pool, PooledConnection},
};

use std::{error, time::Duration};

pub struct DbExecutor {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl DbExecutor {
    pub fn new<S: Into<String>>(url: S) -> Result<DbExecutor> {
        let url = url.into();

        match Pool::builder()
            .connection_timeout(Duration::from_secs(60))
            .error_handler(Box::new(R2d2ErrorHandler))
            .build(ConnectionManager::<PgConnection>::new(&url))
        {
            Ok(pool) => Ok(DbExecutor { pool }),
            Err(err) => Err(Error::PgConnPoolError(err)),
        }
    }

    pub fn get_connection(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>> {
        Ok(self.pool.get().map_err(|err| Error::PgConnPoolError(err))?)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct R2d2ErrorHandler;

impl<E> HandleError<E> for R2d2ErrorHandler
where
    E: error::Error,
{
    fn handle_error(&self, _error: E) {}
}
