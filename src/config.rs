use crate::{
    general::GeneralConfig, logger::LoggerConfig, postgres::PostgresConfig, redis::RedisConfig,
};
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub logger: LoggerConfig,
    pub postgres: PostgresConfig,
    pub redis: RedisConfig,
}

pub fn load_config() -> Result<Config> {
    let config = ::config::Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()?
        .try_deserialize()?;
    Ok(config)
}
