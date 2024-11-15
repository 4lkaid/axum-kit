use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

static REDIS: OnceLock<redis::Client> = OnceLock::new();

pub async fn init(config: &RedisConfig) -> Result<()> {
    let client = redis::Client::open(config.url.as_str())?;
    client.get_multiplexed_tokio_connection().await?;
    REDIS
        .set(client)
        .map_err(|_| anyhow!("Failed to set OnceLock<redis::Client>"))
}

pub async fn conn() -> Result<redis::aio::MultiplexedConnection> {
    Ok(REDIS
        .get()
        .ok_or_else(|| anyhow!("OnceLock<redis::Client> not initialized"))?
        .get_multiplexed_tokio_connection()
        .await?)
}
