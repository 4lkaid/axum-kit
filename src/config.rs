use crate::{
    general::GeneralConfig, logger::LoggerConfig, postgres::PostgresConfig, redis::RedisConfig,
};
use anyhow::Result;
use serde::Deserialize;

macro_rules! deserialize_with_context {
    ($name:ident, $type:ty, $context:expr) => {
        fn $name<'de, D>(deserializer: D) -> std::result::Result<$type, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let config = <$type>::deserialize(deserializer)
                .map_err(|e| serde::de::Error::custom(format!("{}: {}", $context, e)))?;
            Ok(config)
        }
    };
}

deserialize_with_context!(deserialize_general_config, GeneralConfig, "[general]");
deserialize_with_context!(deserialize_logger_config, LoggerConfig, "[logger]");
deserialize_with_context!(deserialize_postgres_config, PostgresConfig, "[postgres]");
deserialize_with_context!(deserialize_redis_config, RedisConfig, "[redis]");

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_general_config")]
    pub general: GeneralConfig,
    #[serde(deserialize_with = "deserialize_logger_config")]
    pub logger: LoggerConfig,
    #[serde(deserialize_with = "deserialize_postgres_config")]
    pub postgres: PostgresConfig,
    #[serde(deserialize_with = "deserialize_redis_config")]
    pub redis: RedisConfig,
}

pub fn load_config() -> Result<Config> {
    let config = ::config::Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()?
        .try_deserialize()?;
    Ok(config)
}
