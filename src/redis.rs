use anyhow::{anyhow, Result};
use bb8_redis::{bb8, RedisConnectionManager};
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

static REDIS_POOL: OnceLock<bb8::Pool<RedisConnectionManager>> = OnceLock::new();

pub async fn init(config: &RedisConfig) -> Result<()> {
    let manager = RedisConnectionManager::new(config.url.as_str())?;
    let pool = bb8::Pool::builder().build(manager).await?;
    REDIS_POOL
        .set(pool)
        .map_err(|_| anyhow!("Failed to set OnceLock<RedisPool>"))
}

pub async fn conn() -> Result<bb8::PooledConnection<'static, RedisConnectionManager>> {
    Ok(REDIS_POOL
        .get()
        .ok_or_else(|| anyhow!("OnceLock<RedisPool> not initialized"))?
        .get()
        .await?)
}
