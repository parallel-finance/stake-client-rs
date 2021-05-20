use crate::error::Result;
use serde::Deserialize;

use std::{fs::read_to_string, path::Path};

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct PostgresConfig {
    // The postgres db username
    pub username: String,

    // The postgres db password
    pub password: String,

    // The postgres server HOST
    pub host: String,

    // The postgres db for this application
    pub db: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub postgres: PostgresConfig,
}

impl Config {
    pub fn get_postgres_url(&self) -> String {
        format!("postgres://{}:{}@{}/{}", self.postgres.username, self.postgres.password, self.postgres.host, self.postgres.db)
    }

    pub fn from_file<P: AsRef<Path>>(p: P) ->  Result<Config> {
        Ok(toml::from_str(&read_to_string(p)?)?)
    }
}
