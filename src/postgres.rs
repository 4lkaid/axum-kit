use anyhow::Result;
use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, Executor, PgPool};
use std::{sync::OnceLock, time::Duration};

#[derive(Debug, Deserialize)]
pub struct PostgresConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
}

static PG_POOL: OnceLock<PgPool> = OnceLock::new();

pub async fn init(config: &PostgresConfig) -> Result<()> {
    let pool = PgPoolOptions::new()
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                conn.execute("SET TIME ZONE 'Asia/Shanghai';").await?;
                Ok(())
            })
        })
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.acquire_timeout))
        .idle_timeout(Some(Duration::from_secs(config.idle_timeout)))
        .max_lifetime(Some(Duration::from_secs(config.max_lifetime)))
        .connect(config.url.as_str())
        .await?;
    PG_POOL
        .set(pool)
        .map_err(|_| anyhow::anyhow!("Failed to set OnceLock<PgPool>"))
}

pub fn conn() -> &'static PgPool {
    PG_POOL.get().expect("OnceLock<PgPool> not initialized")
}
