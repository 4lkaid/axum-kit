#[cfg(feature = "postgres")]
use crate::postgres;

#[cfg(feature = "redis")]
use crate::redis;

use crate::{
    config::{load_config, Config},
    general, logger,
};
use anyhow::{Context, Result};
use axum::Router;
use tracing_appender::non_blocking::WorkerGuard;

type TaskHandle = tokio::task::JoinHandle<Result<()>>;

pub struct Application {
    config: Config,
    router_fn: Option<Box<dyn FnOnce() -> Router + Send + Sync>>,
    pre_run_fn: Option<Box<dyn FnOnce() -> TaskHandle + Send + Sync>>,
}

impl Application {
    pub fn default(config_path: &str) -> Result<Self> {
        let config = load_config(config_path).with_context(|| "configuration parsing failed")?;
        Ok(Self::new(config))
    }

    pub fn new(config: Config) -> Self {
        Self {
            config,
            router_fn: None,
            pre_run_fn: None,
        }
    }

    pub fn with_router<F>(mut self, callback: F) -> Self
    where
        F: FnOnce() -> Router + Send + Sync + 'static,
    {
        self.router_fn = Some(Box::new(callback));
        self
    }

    pub fn before_run<F>(mut self, callback: F) -> Self
    where
        F: FnOnce() -> TaskHandle + Send + Sync + 'static,
    {
        self.pre_run_fn = Some(Box::new(callback));
        self
    }

    pub async fn run(self) -> Result<WorkerGuard> {
        #[cfg(feature = "postgres")]
        postgres::init(&self.config.postgres)
            .await
            .with_context(|| "postgres initialization failed")?;

        #[cfg(feature = "redis")]
        redis::init(&self.config.redis)
            .await
            .with_context(|| "redis initialization failed")?;

        if let Some(callback) = self.pre_run_fn {
            let _ = callback().await?;
        }
        let worker_guard =
            logger::init(&self.config.logger).with_context(|| "logger initialization failed")?;
        let router = self
            .router_fn
            .map(|callback| callback())
            .unwrap_or_else(|| {
                Router::new().route("/", axum::routing::get(|| async { "Hello, Axum-kit!" }))
            });
        general::serve(&self.config.general, router)
            .await
            .with_context(|| "service startup failed")?;

        Ok(worker_guard)
    }
}
