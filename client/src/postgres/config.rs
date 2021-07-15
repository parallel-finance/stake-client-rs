use crate::common::error::Result;

use db::executor::DbExecutor;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::{fs::read_to_string, path::Path};

lazy_static! {
    pub static ref CFG: Config =
        Config::from_file("Config.toml").unwrap_or_else(|_| std::process::exit(1));
    pub static ref DB: DbExecutor = {
        let url = CFG.get_postgres_url();
        DbExecutor::new(&url).unwrap_or_else(|err| {
            println!("exit err:{:?}", err);
            std::process::exit(1)
        })
    };
}

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
        format!(
            "postgres://{}:{}@{}/{}",
            self.postgres.username, self.postgres.password, self.postgres.host, self.postgres.db
        )
    }

    pub fn from_file<P: AsRef<Path>>(p: P) -> Result<Config> {
        Ok(toml::from_str(&read_to_string(p)?)?)
    }
}
